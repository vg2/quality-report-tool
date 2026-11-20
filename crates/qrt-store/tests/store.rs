#![allow(clippy::unwrap_used, clippy::expect_used)]

use qrt_domain::{Name, WorkspaceId};
use qrt_secrets::{DbKey, MemoryStore, SecretStore, REGISTRY_KEY_ACCOUNT};
use qrt_store::{open_encrypted, SqliteStore, StoreError, SystemRepository, WorkspaceRegistry};
use std::sync::Arc;

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

#[test]
fn workspace_and_system_round_trip_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());

    let ws_id = {
        let store = SqliteStore::open(dir.path(), secrets.clone()).unwrap();
        let ws = store.create_workspace(name("Acme assessment")).unwrap();
        let db = store.open_workspace(ws.id).unwrap();
        db.create_system(name("Customer portal"), Some("Public web app".into()))
            .unwrap();
        ws.id
    };

    let store = SqliteStore::open(dir.path(), secrets).unwrap();
    let list = store.list_workspaces().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name.as_str(), "Acme assessment");

    let systems = store.open_workspace(ws_id).unwrap().list_systems().unwrap();
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].name.as_str(), "Customer portal");
    assert_eq!(systems[0].description.as_deref(), Some("Public web app"));
}

#[test]
fn database_files_are_not_plaintext_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(dir.path(), Arc::new(MemoryStore::default())).unwrap();
    let ws = store.create_workspace(name("Secret client name")).unwrap();
    drop(store);

    for path in [
        dir.path().join("registry.db"),
        dir.path().join("workspaces").join(format!("{}.db", ws.id)),
    ] {
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            !bytes.starts_with(b"SQLite format 3"),
            "{path:?} is plaintext"
        );
        let needle = b"Secret client name";
        assert!(
            !bytes.windows(needle.len()).any(|w| w == needle),
            "{path:?} leaks data"
        );
    }
}

#[test]
fn wrong_key_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemoryStore::default());
    SqliteStore::open(dir.path(), secrets.clone()).unwrap();

    let wrong = DbKey::generate().unwrap();
    let result = open_encrypted(&dir.path().join("registry.db"), &wrong);
    assert!(matches!(result, Err(StoreError::WrongKeyOrCorrupt)));

    // Replacing the keychain entry also prevents the store from opening.
    secrets.set(REGISTRY_KEY_ACCOUNT, &wrong.to_hex()).unwrap();
    assert!(matches!(
        SqliteStore::open(dir.path(), secrets),
        Err(StoreError::WrongKeyOrCorrupt)
    ));
}

#[test]
fn unknown_workspace_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteStore::open(dir.path(), Arc::new(MemoryStore::default())).unwrap();
    assert!(matches!(
        store.open_workspace(WorkspaceId::new()),
        Err(StoreError::NotFound)
    ));
}

#[cfg(unix)]
#[test]
fn files_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    SqliteStore::open(dir.path(), Arc::new(MemoryStore::default())).unwrap();
    let mode = std::fs::metadata(dir.path().join("registry.db"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}
