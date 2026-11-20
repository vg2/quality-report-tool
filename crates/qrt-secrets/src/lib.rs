//! Secret storage and database key management.
//!
//! Secrets (database keys, PATs, API keys) live only in the OS keychain —
//! never in a database, config file or log. See `docs/plan.md` §8.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

/// Keychain service name under which all of this app's secrets are stored.
pub const SERVICE: &str = "com.qualityreport.tool";

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("secret store unavailable: {0}")]
    Unavailable(String),
    #[error("stored secret is malformed")]
    Malformed,
    #[error("random number generator failed")]
    Rng,
}

/// Abstraction over a secret store so tests can run without a keychain.
pub trait SecretStore: Send + Sync {
    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>, SecretError>;
    fn set(&self, account: &str, secret: &str) -> Result<(), SecretError>;
    fn delete(&self, account: &str) -> Result<(), SecretError>;
}

/// OS keychain: macOS Keychain, Windows Credential Manager, Secret Service.
#[derive(Debug, Default)]
pub struct KeyringStore;

impl KeyringStore {
    fn entry(account: &str) -> Result<keyring::Entry, SecretError> {
        keyring::Entry::new(SERVICE, account).map_err(|e| SecretError::Unavailable(e.to_string()))
    }
}

impl SecretStore for KeyringStore {
    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        match Self::entry(account)?.get_password() {
            Ok(s) => Ok(Some(Zeroizing::new(s))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretError::Unavailable(e.to_string())),
        }
    }

    fn set(&self, account: &str, secret: &str) -> Result<(), SecretError> {
        Self::entry(account)?
            .set_password(secret)
            .map_err(|e| SecretError::Unavailable(e.to_string()))
    }

    fn delete(&self, account: &str) -> Result<(), SecretError> {
        match Self::entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(SecretError::Unavailable(e.to_string())),
        }
    }
}

/// In-memory store for tests and ephemeral use.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<HashMap<String, Zeroizing<String>>>,
}

impl SecretStore for MemoryStore {
    fn get(&self, account: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        let map = self
            .inner
            .lock()
            .map_err(|_| SecretError::Unavailable("poisoned".into()))?;
        Ok(map.get(account).cloned())
    }

    fn set(&self, account: &str, secret: &str) -> Result<(), SecretError> {
        let mut map = self
            .inner
            .lock()
            .map_err(|_| SecretError::Unavailable("poisoned".into()))?;
        map.insert(account.to_owned(), Zeroizing::new(secret.to_owned()));
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<(), SecretError> {
        let mut map = self
            .inner
            .lock()
            .map_err(|_| SecretError::Unavailable("poisoned".into()))?;
        map.remove(account);
        Ok(())
    }
}

/// A 256-bit raw database encryption key. Zeroed on drop; never printed.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct DbKey([u8; 32]);

impl DbKey {
    pub fn generate() -> Result<Self, SecretError> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| SecretError::Rng)?;
        Ok(Self(bytes))
    }

    pub fn from_hex(s: &str) -> Result<Self, SecretError> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes).map_err(|_| SecretError::Malformed)?;
        Ok(Self(bytes))
    }

    pub fn to_hex(&self) -> Zeroizing<String> {
        Zeroizing::new(hex::encode(self.0))
    }
}

impl fmt::Debug for DbKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DbKey(<redacted>)")
    }
}

/// Load the key for `account`, creating and storing a new random key if
/// none exists yet.
pub fn get_or_create_db_key(store: &dyn SecretStore, account: &str) -> Result<DbKey, SecretError> {
    if let Some(hex) = store.get(account)? {
        return DbKey::from_hex(&hex);
    }
    let key = DbKey::generate()?;
    store.set(account, &key.to_hex())?;
    Ok(key)
}

/// Keychain account name for the app-level registry database key.
pub const REGISTRY_KEY_ACCOUNT: &str = "db-key:registry";

/// Keychain account name for a workspace database key.
pub fn workspace_key_account(workspace_id: &str) -> String {
    format!("db-key:workspace:{workspace_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_then_reuses_key() -> Result<(), SecretError> {
        let store = MemoryStore::default();
        let a = get_or_create_db_key(&store, "acct")?;
        let b = get_or_create_db_key(&store, "acct")?;
        assert_eq!(*a.to_hex(), *b.to_hex());
        Ok(())
    }

    #[test]
    fn generated_keys_differ() -> Result<(), SecretError> {
        assert_ne!(*DbKey::generate()?.to_hex(), *DbKey::generate()?.to_hex());
        Ok(())
    }

    #[test]
    fn debug_does_not_leak_key() -> Result<(), SecretError> {
        let key = DbKey::generate()?;
        let printed = format!("{key:?}");
        assert!(!printed.contains(key.to_hex().as_str()));
        Ok(())
    }

    #[test]
    fn rejects_malformed_hex() {
        assert!(matches!(DbKey::from_hex("zz"), Err(SecretError::Malformed)));
    }
}
