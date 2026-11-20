#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("secret store error: {0}")]
    Secret(#[from] qrt_secrets::SecretError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database could not be decrypted (wrong key or corrupt file)")]
    WrongKeyOrCorrupt,
    #[error("not found")]
    NotFound,
    #[error("stored data is invalid: {0}")]
    Corrupt(String),
    #[error("database schema version {found} is newer than this app supports ({supported})")]
    SchemaTooNew { found: i64, supported: i64 },
    #[error("lock poisoned")]
    Poisoned,
}
