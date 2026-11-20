//! Core domain types. This crate performs no IO; it holds entities,
//! identifiers and validated value types shared across the app.

mod ids;
mod name;

pub use ids::{SystemId, WorkspaceId};
pub use name::{Name, NameError};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// A workspace is one encrypted database holding branding and one or more
/// assessed systems (see `docs/plan.md` §2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: Name,
    pub created_at: Timestamp,
}

impl Workspace {
    pub fn new(name: Name) -> Self {
        Self {
            id: WorkspaceId::new(),
            name,
            created_at: Timestamp::now(),
        }
    }
}

/// The software system being assessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct System {
    pub id: SystemId,
    pub name: Name,
    pub description: Option<String>,
    pub created_at: Timestamp,
}

impl System {
    pub fn new(name: Name, description: Option<String>) -> Self {
        Self {
            id: SystemId::new(),
            name,
            description,
            created_at: Timestamp::now(),
        }
    }
}
