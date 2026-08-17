use crate::schema::GrantScope;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct GrantStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Grant {
    pub id: String,
    pub tool_class: String,
    pub tool_name: String,
    pub shape_hash: String,
    pub scope: GrantScope,
    pub run_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl GrantStore {
    /// # Errors
    ///
    /// Returns an error if the `SQLite` database at `path` cannot be opened,
    /// or if the `grants` table/index cannot be created.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init_tables(&conn)?;
        Ok(Self { conn })
    }

    /// # Errors
    ///
    /// Returns an error if the in-memory `SQLite` database cannot be opened,
    /// or if the `grants` table/index cannot be created.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_tables(&conn)?;
        Ok(Self { conn })
    }

    fn init_tables(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS grants (
                id TEXT PRIMARY KEY,
                tool_class TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                shape_hash TEXT NOT NULL,
                scope TEXT NOT NULL,
                run_id TEXT,
                expires_at INTEGER,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;
        // Shape matching uses tool_name + shape_hash.
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_grants_shape ON grants(tool_name, shape_hash)",
            [],
        )?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the underlying `SQLite` `INSERT` fails.
    pub fn add_grant(&mut self, grant: &Grant) -> Result<()> {
        let expires_at_timestamp = grant.expires_at.map(|t| t.timestamp());
        let created_at_timestamp = Utc::now().timestamp();

        let scope_str = match grant.scope {
            GrantScope::Once => "once",
            GrantScope::Run => "run",
            GrantScope::Session => "session",
            GrantScope::Always => "always",
        };

        self.conn.execute(
            "INSERT INTO grants (id, tool_class, tool_name, shape_hash, scope, run_id, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                grant.id,
                grant.tool_class,
                grant.tool_name,
                grant.shape_hash,
                scope_str,
                grant.run_id,
                expires_at_timestamp,
                created_at_timestamp
            ],
        )?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the underlying `SQLite` query fails to prepare,
    /// execute, or map its result rows.
    pub fn check_grant(
        &self,
        tool_name: &str,
        shape_hash: &str,
        current_run_id: Option<&str>,
    ) -> Result<Option<Grant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tool_class, tool_name, shape_hash, scope, run_id, expires_at
             FROM grants 
             WHERE tool_name = ?1 AND shape_hash = ?2",
        )?;

        let _now = Utc::now().timestamp();

        let grants = stmt.query_map(params![tool_name, shape_hash], |row| {
            let id: String = row.get(0)?;
            let tool_class: String = row.get(1)?;
            let t_name: String = row.get(2)?;
            let s_hash: String = row.get(3)?;
            let scope_str: String = row.get(4)?;
            let run_id: Option<String> = row.get(5)?;
            let expires_at_ts: Option<i64> = row.get(6)?;

            let scope = match scope_str.as_str() {
                "run" => GrantScope::Run,
                "session" => GrantScope::Session,
                "always" => GrantScope::Always,
                _ => GrantScope::Once,
            };

            let expires_at = expires_at_ts.and_then(|ts| Utc.timestamp_opt(ts, 0).single());

            Ok(Grant {
                id,
                tool_class,
                tool_name: t_name,
                shape_hash: s_hash,
                scope,
                run_id,
                expires_at,
            })
        })?;

        for grant in grants.flatten() {
            // Check expiry
            if let Some(exp) = grant.expires_at
                && Utc::now() > exp
            {
                continue; // Expired
            }

            // Check scope constraints
            if grant.scope == GrantScope::Run && grant.run_id.as_deref() != current_run_id {
                continue;
            }

            // If it's Once, we should probably consume it (handled separately by engine for now or here)
            // For simplicity, we just return it. The engine can revoke it.
            return Ok(Some(grant));
        }

        Ok(None)
    }

    /// # Errors
    ///
    /// Returns an error if the underlying `SQLite` `DELETE` fails.
    pub fn revoke_grant(&mut self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM grants WHERE id = ?1", params![id])?;
        Ok(())
    }
}
