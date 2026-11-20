//! Minimal forward-only migrations tracked with `PRAGMA user_version`.

use crate::StoreError;
use rusqlite::Connection;

pub struct Migration {
    pub version: i64,
    pub sql: &'static str,
}

pub const REGISTRY: &[Migration] = &[Migration {
    version: 1,
    sql: include_str!("migrations/registry_0001_init.sql"),
}];

pub const WORKSPACE: &[Migration] = &[Migration {
    version: 1,
    sql: include_str!("migrations/workspace_0001_init.sql"),
}];

/// Apply all migrations newer than the database's current version, each in
/// its own transaction. Refuses to open a database from a newer app version.
pub fn run(conn: &mut Connection, migrations: &[Migration]) -> Result<(), StoreError> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let latest = migrations.last().map_or(0, |m| m.version);
    if current > latest {
        return Err(StoreError::SchemaTooNew {
            found: current,
            supported: latest,
        });
    }
    for m in migrations.iter().filter(|m| m.version > current) {
        let tx = conn.transaction()?;
        tx.execute_batch(m.sql)?;
        // user_version cannot be bound as a parameter; the value is a
        // compile-time constant, not user input.
        tx.execute_batch(&format!("PRAGMA user_version = {}", m.version))?;
        tx.commit()?;
        tracing::info!(version = m.version, "applied migration");
    }
    Ok(())
}
