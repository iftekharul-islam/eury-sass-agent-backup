use keyring::Entry;
use rand::Rng;
use rusqlite::Connection;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("Migration error: {0}")]
    Migration(String),
}

pub struct Store {
    pub conn: Connection,
}

impl Store {
    /// # Errors
    ///
    /// Returns [`StoreError::Keyring`] if the encryption key cannot be
    /// read/created, or [`StoreError::Database`]/[`StoreError::Migration`]
    /// if the database cannot be opened, verified, or quarantined.
    pub fn open_or_create(path: &str) -> Result<Self, StoreError> {
        let key = Self::get_or_create_key_for(Some(path))?;
        Self::open_or_create_with_key(path, &key)
    }

    /// Opens the store with an explicitly supplied `SQLCipher` key instead of
    /// reading one from the OS keychain.
    ///
    /// Intended for tests and headless environments: CI runners have no
    /// keychain daemon, and on macOS each freshly built, unsigned test binary
    /// triggers an interactive keychain prompt a non-interactive run cannot
    /// answer. The shipped app uses [`Self::open_or_create`].
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Database`]/[`StoreError::Migration`] if the
    /// database cannot be opened, verified, or quarantined.
    pub fn open_or_create_with_key(path: &str, key: &str) -> Result<Self, StoreError> {
        // Attempt to open and verify
        if let Ok(conn) = Self::open_and_verify(path, key) {
            Ok(Self { conn })
        } else {
            // Verification failed, likely corrupted. Quarantine.
            if std::path::Path::new(path).exists() {
                let timestamp = chrono::Utc::now().timestamp();
                let quarantine_path = format!("{path}.corrupt-{timestamp}");
                std::fs::rename(path, &quarantine_path)
                    .map_err(|e| StoreError::Migration(format!("Failed to quarantine DB: {e}")))?;
            }

            // Try again with fresh DB
            let conn = Self::open_and_verify(path, key)?;
            Ok(Self { conn })
        }
    }

    /// Reads (or creates) the dev key file next to the database.
    ///
    /// `PathGuard` exists to contain agent- and tool-supplied paths to
    /// workspace content. This path is the store's own database path with a
    /// `.key` suffix — never agent-supplied — which is the same reasoning that
    /// exempts this crate in `scripts/check-boundaries.mjs`.
    ///
    /// The file is created with `0600` in the same call that creates it, so it
    /// is never briefly readable by other users, and `create_new` makes two
    /// simultaneous first launches safe: the loser reads the winner's key
    /// rather than overwriting it and orphaning the database.
    #[allow(clippy::disallowed_types, clippy::disallowed_methods)]
    fn key_from_sidecar_file(db_path: &str) -> Result<String, StoreError> {
        use std::io::{Read as _, Write as _};

        let key_path = std::path::PathBuf::from(format!("{db_path}.key"));

        let read_existing = |path: &std::path::Path| -> Option<String> {
            let mut file = std::fs::File::open(path).ok()?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).ok()?;
            let trimmed = buffer.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        };

        if let Some(existing) = read_existing(&key_path) {
            return Ok(existing);
        }

        let mut key_bytes = [0u8; 32];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut key_bytes);
        let key_hex = hex::encode(key_bytes);

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }

        match options.open(&key_path) {
            Ok(mut file) => {
                file.write_all(key_hex.as_bytes()).map_err(|e| {
                    StoreError::Migration(format!("failed to write dev key file: {e}"))
                })?;
                Ok(key_hex)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read_existing(&key_path)
                .ok_or_else(|| {
                    StoreError::Migration("dev key file exists but is unreadable".to_string())
                }),
            Err(e) => Err(StoreError::Migration(format!("failed to create dev key file: {e}"))),
        }
    }

    fn open_and_verify(path: &str, key: &str) -> Result<Connection, StoreError> {
        let conn = Connection::open(path)?;

        // Setup encryption
        conn.pragma_update(None, "key", key)?;

        // Optimizations and correctness PRAGMAs
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        conn.pragma_update(None, "mmap_size", 268_435_456)?; // 256 MB
        conn.pragma_update(None, "cache_size", -16000)?; // 16 MB
        conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")?;
        conn.pragma_update(None, "wal_autocheckpoint", 1000)?;

        // Verify integrity
        {
            let mut stmt = conn.prepare("PRAGMA integrity_check")?;
            let result: String = stmt.query_row([], |row| row.get(0))?;
            if result != "ok" {
                return Err(StoreError::Migration("Database integrity check failed".to_string()));
            }
        }

        Ok(conn)
    }

    /// Set `EURY_AGENT_DB_KEY` to supply the `SQLCipher` key directly instead
    /// of reading it from the OS keychain.
    ///
    /// This exists because the keychain is not always reachable: CI runners
    /// are headless (no keychain daemon at all), and on macOS each freshly
    /// built, unsigned test binary triggers an interactive access prompt that
    /// a non-interactive run can only fail. Mirrors `EURY_AGENT_AUTH_STORE`
    /// in `agent-core`. Not for production use — the shipped app reads the
    /// key from the keychain.
    const KEY_ENV_VAR: &'static str = "EURY_AGENT_DB_KEY";

    /// The key for the database at `db_path`.
    ///
    /// Release builds keep the key in the OS keychain. Debug builds keep it in
    /// a `0600` file beside the database instead: the keychain binds an item's
    /// ACL to the calling binary's code signature, so every rebuild of an
    /// ad-hoc-signed dev binary is a different app to macOS and the user is
    /// asked for their login password on every single launch. An explicit
    /// `EURY_AGENT_DB_KEY` still wins over both.
    fn get_or_create_key_for(db_path: Option<&str>) -> Result<String, StoreError> {
        if let Ok(key) = std::env::var(Self::KEY_ENV_VAR)
            && !key.is_empty()
        {
            return Ok(key);
        }

        if cfg!(debug_assertions)
            && let Some(path) = db_path
        {
            return Self::key_from_sidecar_file(path);
        }

        let entry = Entry::new("com.eury.agent", "db-key")?;

        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(keyring::Error::NoEntry) => {
                let mut key_bytes = [0u8; 32];
                // Note: Using OsRng via seed for more secure generation if available, fallback otherwise
                let mut rng = rand::rng();
                rng.fill_bytes(&mut key_bytes);
                let key_hex = hex::encode(key_bytes);

                // `get_password` → `set_password` is a check-then-act race:
                // two first-launch opens can both see `NoEntry` and both try
                // to create the key. The loser must adopt the winner's key,
                // never overwrite it — overwriting would orphan a database
                // already encrypted under the winner's key, which reads as
                // corruption and triggers the quarantine path below.
                match entry.set_password(&key_hex) {
                    Ok(()) => Ok(key_hex),
                    Err(e) => entry.get_password().map_err(|_| StoreError::Keyring(e)),
                }
            }
            Err(e) => Err(StoreError::Keyring(e)),
        }
    }
}
