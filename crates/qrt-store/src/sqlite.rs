use crate::{migrate, StoreError, SystemRepository, WorkspaceRegistry};
use jiff::Timestamp;
use qrt_domain::{Name, System, SystemId, Workspace, WorkspaceId};
use qrt_secrets::{
    get_or_create_db_key, workspace_key_account, DbKey, SecretStore, REGISTRY_KEY_ACCOUNT,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const REGISTRY_FILE: &str = "registry.db";
const WORKSPACES_DIR: &str = "workspaces";

/// Open (or create) a SQLCipher database at `path` using a raw 256-bit key,
/// and verify the key is correct.
pub fn open_encrypted(path: &Path, key: &DbKey) -> Result<Connection, StoreError> {
    let conn = Connection::open(path)?;
    // Raw-key form skips PBKDF2 (the key is already random and full-entropy).
    // The value is hex produced by us, never user input.
    let hex = key.to_hex();
    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", hex.as_str()))?;
    // Reading the schema fails if the key is wrong or the file is not ours.
    if conn
        .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        })
        .is_err()
    {
        return Err(StoreError::WrongKeyOrCorrupt);
    }
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA secure_delete = ON;
         PRAGMA journal_mode = WAL;",
    )?;
    restrict_permissions(path, false)?;
    Ok(conn)
}

/// Owner-only permissions on files and directories we create.
fn restrict_permissions(path: &Path, is_dir: bool) -> Result<(), StoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if is_dir { 0o700 } else { 0o600 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    let _ = (path, is_dir);
    Ok(())
}

fn ensure_dir(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path)?;
    restrict_permissions(path, true)
}

fn parse_time(s: &str) -> Result<Timestamp, StoreError> {
    s.parse()
        .map_err(|e| StoreError::Corrupt(format!("timestamp: {e}")))
}

fn parse_name(s: &str) -> Result<Name, StoreError> {
    Name::parse(s).map_err(|e| StoreError::Corrupt(format!("name: {e}")))
}

/// App-level store: the encrypted registry of workspaces, plus access to each
/// workspace's own encrypted database.
pub struct SqliteStore {
    root: PathBuf,
    secrets: Arc<dyn SecretStore>,
    registry: Mutex<Connection>,
}

impl SqliteStore {
    /// Open the store rooted at `root` (the app data directory).
    pub fn open(
        root: impl Into<PathBuf>,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<Self, StoreError> {
        let root = root.into();
        ensure_dir(&root)?;
        ensure_dir(&root.join(WORKSPACES_DIR))?;
        let key = get_or_create_db_key(secrets.as_ref(), REGISTRY_KEY_ACCOUNT)?;
        let mut conn = open_encrypted(&root.join(REGISTRY_FILE), &key)?;
        migrate::run(&mut conn, migrate::REGISTRY)?;
        Ok(Self {
            root,
            secrets,
            registry: Mutex::new(conn),
        })
    }

    /// Database path for a workspace. Derived only from the ID, never from
    /// user-supplied text, so it cannot be used for path traversal.
    fn workspace_path(&self, id: WorkspaceId) -> PathBuf {
        self.root.join(WORKSPACES_DIR).join(format!("{id}.db"))
    }

    /// Open a workspace's encrypted database.
    pub fn open_workspace(&self, id: WorkspaceId) -> Result<SqliteWorkspace, StoreError> {
        // Confirms the workspace is registered before touching the filesystem.
        self.get_workspace(id)?;
        let key = self
            .secrets
            .get(&workspace_key_account(&id.to_string()))?
            .ok_or(StoreError::NotFound)
            .and_then(|hex| Ok(DbKey::from_hex(&hex)?))?;
        let mut conn = open_encrypted(&self.workspace_path(id), &key)?;
        migrate::run(&mut conn, migrate::WORKSPACE)?;
        Ok(SqliteWorkspace {
            conn: Mutex::new(conn),
        })
    }

    fn registry(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
        self.registry.lock().map_err(|_| StoreError::Poisoned)
    }
}

impl WorkspaceRegistry for SqliteStore {
    fn list_workspaces(&self) -> Result<Vec<Workspace>, StoreError> {
        let conn = self.registry()?;
        let mut stmt =
            conn.prepare("SELECT id, name, created_at FROM workspaces ORDER BY created_at, id")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        rows.map(|row| {
            let (id, name, created_at) = row?;
            Ok(Workspace {
                id: id
                    .parse()
                    .map_err(|_| StoreError::Corrupt("workspace id".into()))?,
                name: parse_name(&name)?,
                created_at: parse_time(&created_at)?,
            })
        })
        .collect()
    }

    fn get_workspace(&self, id: WorkspaceId) -> Result<Workspace, StoreError> {
        let conn = self.registry()?;
        let row = conn
            .query_row(
                "SELECT name, created_at FROM workspaces WHERE id = ?1",
                params![id.to_string()],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
            .ok_or(StoreError::NotFound)?;
        Ok(Workspace {
            id,
            name: parse_name(&row.0)?,
            created_at: parse_time(&row.1)?,
        })
    }

    fn create_workspace(&self, name: Name) -> Result<Workspace, StoreError> {
        let ws = Workspace::new(name);
        let key = get_or_create_db_key(
            self.secrets.as_ref(),
            &workspace_key_account(&ws.id.to_string()),
        )?;
        let mut conn = open_encrypted(&self.workspace_path(ws.id), &key)?;
        migrate::run(&mut conn, migrate::WORKSPACE)?;
        conn.execute(
            "INSERT INTO meta (key, value) VALUES ('workspace_id', ?1)",
            params![ws.id.to_string()],
        )?;
        drop(conn);
        self.registry()?.execute(
            "INSERT INTO workspaces (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![
                ws.id.to_string(),
                ws.name.as_str(),
                ws.created_at.to_string()
            ],
        )?;
        Ok(ws)
    }
}

/// One workspace's encrypted database.
pub struct SqliteWorkspace {
    conn: Mutex<Connection>,
}

impl SqliteWorkspace {
    fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
        self.conn.lock().map_err(|_| StoreError::Poisoned)
    }
}

impl SystemRepository for SqliteWorkspace {
    fn list_systems(&self) -> Result<Vec<System>, StoreError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at FROM systems ORDER BY created_at, id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        rows.map(|row| {
            let (id, name, description, created_at) = row?;
            Ok(System {
                id: id
                    .parse::<SystemId>()
                    .map_err(|_| StoreError::Corrupt("system id".into()))?,
                name: parse_name(&name)?,
                description,
                created_at: parse_time(&created_at)?,
            })
        })
        .collect()
    }

    fn create_system(&self, name: Name, description: Option<String>) -> Result<System, StoreError> {
        let system = System::new(name, description);
        self.conn()?.execute(
            "INSERT INTO systems (id, name, description, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                system.id.to_string(),
                system.name.as_str(),
                system.description,
                system.created_at.to_string()
            ],
        )?;
        Ok(system)
    }
}
