//! Persistence for the quality report tool.
//!
//! * Repository **traits** define what the app needs from storage, so other
//!   backends (Postgres, Jira, Azure DevOps) can be added later.
//! * The default implementation is SQLCipher-encrypted SQLite: one app-level
//!   *registry* database listing workspaces, and one database per workspace.
//!   Each database has its own random 256-bit key held in the OS keychain.

mod error;
mod migrate;
mod sqlite;

pub use error::StoreError;
pub use sqlite::{open_encrypted, SqliteStore, SqliteWorkspace};

use qrt_domain::{Name, System, Workspace, WorkspaceId};

/// Lists and creates workspaces.
pub trait WorkspaceRegistry {
    fn list_workspaces(&self) -> Result<Vec<Workspace>, StoreError>;
    fn get_workspace(&self, id: WorkspaceId) -> Result<Workspace, StoreError>;
    fn create_workspace(&self, name: Name) -> Result<Workspace, StoreError>;
}

/// Systems assessed within one workspace.
pub trait SystemRepository {
    fn list_systems(&self) -> Result<Vec<System>, StoreError>;
    fn create_system(&self, name: Name, description: Option<String>) -> Result<System, StoreError>;
}
