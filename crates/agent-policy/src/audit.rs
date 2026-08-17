use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Result, params};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

pub struct AuditQueue {
    conn: Connection,
}

/// One stored audit row, as returned by [`AuditQueue::list_events`].
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub seq: i64,
    pub event_type: String,
    pub severity: String,
    pub run_id: String,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub id: String,
    pub seq: i64,
    pub prev_hash: String,
    pub event_type: String,
    pub severity: String,
    pub occurred_at: String,
    pub user_id: String,
    pub organization_id: String,
    pub device_id: String,
    pub run_id: String,
    pub policy_version: u32,
    pub app_version: String,
    pub workspace_hash: String,
    pub payload: Value,
}

impl AuditEvent {
    #[must_use]
    pub fn new(event_type: String, payload: Value) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            seq: 0,
            prev_hash: String::new(),
            event_type,
            severity: "info".to_string(),
            occurred_at: Utc::now().to_rfc3339(),
            user_id: "system".to_string(),
            organization_id: "local".to_string(),
            device_id: "local".to_string(),
            run_id: "none".to_string(),
            policy_version: 1,
            app_version: "1.0.0".to_string(),
            workspace_hash: "mock".to_string(),
            payload,
        }
    }
}

impl AuditQueue {
    /// # Errors
    ///
    /// Returns an error if the `SQLite` database at `path` cannot be opened,
    /// or if the `audit_events` table cannot be created.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::init_tables(&conn)?;
        Ok(Self { conn })
    }

    /// # Errors
    ///
    /// Returns an error if the in-memory `SQLite` database cannot be opened,
    /// or if the `audit_events` table cannot be created.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_tables(&conn)?;
        Ok(Self { conn })
    }

    fn init_tables(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                seq INTEGER NOT NULL,
                prev_hash TEXT NOT NULL,
                event_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                user_id TEXT NOT NULL,
                organization_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                policy_version INTEGER NOT NULL,
                app_version TEXT NOT NULL,
                workspace_hash TEXT NOT NULL,
                payload TEXT NOT NULL,
                hash TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the underlying `SQLite` transaction fails to start,
    /// query the previous chain entry, insert the new event, or commit.
    pub fn append_event(&mut self, mut event: AuditEvent) -> Result<()> {
        let tx = self.conn.transaction()?;

        let (seq, prev_hash) = {
            let mut stmt =
                tx.prepare("SELECT seq, hash FROM audit_events ORDER BY seq DESC LIMIT 1")?;
            let last_record = stmt
                .query_row([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
                .optional()?;

            match last_record {
                Some((s, h)) => (s + 1, h),
                None => (1, "genesis".to_string()),
            }
        };

        event.seq = seq;
        prev_hash.clone_into(&mut event.prev_hash);

        let payload_str = event.payload.to_string();

        let mut hasher = Sha256::new();
        hasher.update(event.id.as_bytes());
        hasher.update(seq.to_le_bytes());
        hasher.update(prev_hash.as_bytes());
        hasher.update(event.event_type.as_bytes());
        hasher.update(payload_str.as_bytes());
        let hash_bytes = hasher.finalize();
        let hash = format!("sha256:{}", hex::encode(hash_bytes));

        tx.execute(
            "INSERT INTO audit_events (
                id, seq, prev_hash, event_type, severity, occurred_at, user_id, 
                organization_id, device_id, run_id, policy_version, app_version, 
                workspace_hash, payload, hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                event.id,
                event.seq,
                event.prev_hash,
                event.event_type,
                event.severity,
                event.occurred_at,
                event.user_id,
                event.organization_id,
                event.device_id,
                event.run_id,
                event.policy_version,
                event.app_version,
                event.workspace_hash,
                payload_str,
                hash
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Returns every recorded event, oldest first. Intended for
    /// verification and export.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `SQLite` query fails.
    pub fn list_events(&self) -> Result<Vec<AuditRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT seq, event_type, severity, run_id, payload FROM audit_events ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let payload_str: String = row.get(4)?;
            Ok(AuditRecord {
                seq: row.get(0)?,
                event_type: row.get(1)?,
                severity: row.get(2)?,
                run_id: row.get(3)?,
                payload: serde_json::from_str(&payload_str).unwrap_or(Value::Null),
            })
        })?;
        rows.collect()
    }

    /// Recomputes the SHA-256 chain over every stored event and reports
    /// whether it is intact — i.e. no row has been edited, reordered, or
    /// removed since it was appended. This is what makes the log
    /// tamper-evident rather than merely append-only.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `SQLite` query fails.
    pub fn verify_chain(&self) -> Result<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT id, seq, prev_hash, event_type, payload, hash
             FROM audit_events ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        let mut expected_prev = "genesis".to_string();
        for (expected_seq, row) in (1i64..).zip(rows) {
            let (id, seq, prev_hash, event_type, payload, hash) = row?;
            if seq != expected_seq || prev_hash != expected_prev {
                return Ok(false);
            }

            let mut hasher = Sha256::new();
            hasher.update(id.as_bytes());
            hasher.update(seq.to_le_bytes());
            hasher.update(prev_hash.as_bytes());
            hasher.update(event_type.as_bytes());
            hasher.update(payload.as_bytes());
            let recomputed = format!("sha256:{}", hex::encode(hasher.finalize()));
            if recomputed != hash {
                return Ok(false);
            }

            expected_prev = hash;
        }

        Ok(true)
    }
}
