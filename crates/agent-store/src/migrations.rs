use crate::db::StoreError;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

#[must_use]
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration { version: 1, name: "initial_schema", sql: crate::schema::V1_SCHEMA },
        Migration {
            version: 2,
            name: "workspace_trust_state",
            sql: crate::schema::V2_WORKSPACE_TRUST,
        },
    ]
}

/// # Errors
///
/// Returns [`StoreError`] if the schema-version bookkeeping table cannot be
/// created, a pre-migration backup copy fails, or a migration's SQL fails to
/// apply.
pub fn run_migrations(conn: &mut Connection, db_path: &Path) -> Result<(), StoreError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL,
            duration_ms INTEGER NOT NULL
        )",
        [],
    )?;

    let current_version: i32 = {
        let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_migrations")?;
        stmt.query_row([], |row| row.get(0)).unwrap_or(0)
    };

    let migrations = get_migrations();

    for m in migrations {
        if m.version > current_version {
            // Snapshot before migrating
            if db_path.exists() {
                let backup_path = db_path.with_extension(format!("db.pre-{}", m.version));
                fs::copy(db_path, backup_path).map_err(|e| StoreError::Migration(e.to_string()))?;
            }

            let start = std::time::Instant::now();
            let tx = conn.transaction()?;

            // foreign_keys must be off during structural migrations, but let's just keep it simple for now
            tx.execute_batch(m.sql)?;

            let checksum = hex::encode(Sha256::digest(m.sql.as_bytes()));

            tx.execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at, duration_ms) 
                 VALUES (?1, ?2, ?3, datetime('now'), ?4)",
                (
                    m.version,
                    m.name,
                    checksum,
                    i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX),
                ),
            )?;

            tx.commit()?;
        }
    }

    Ok(())
}
