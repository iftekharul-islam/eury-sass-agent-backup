// Export/import targets an app-data-dir/user-chosen backup path, never an
// agent- or tool-supplied workspace path — the threat `PathGuard` (see
// scripts/check-boundaries.mjs, which exempts this crate) exists to contain
// does not apply here. Direct `std::fs::File` use is intentional and
// reviewed.
#![allow(clippy::disallowed_types)]

use crate::db::StoreError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    workspaces: Vec<serde_json::Value>,
    conversations: Vec<serde_json::Value>,
    messages: Vec<serde_json::Value>,
    runs: Vec<serde_json::Value>,
    run_steps: Vec<serde_json::Value>,
    settings: Vec<serde_json::Value>,
    grants: Vec<serde_json::Value>,
    audit_queue: Vec<serde_json::Value>,
}

/// # Errors
///
/// Returns [`StoreError`] if any table query fails or the JSON file cannot
/// be created/written.
pub fn export_to_json(conn: &Connection, path: &Path) -> Result<(), StoreError> {
    let data = ExportData {
        workspaces: fetch_table_as_json(conn, "workspaces")?,
        conversations: fetch_table_as_json(conn, "conversations")?,
        messages: fetch_table_as_json(conn, "messages")?,
        runs: fetch_table_as_json(conn, "runs")?,
        run_steps: fetch_table_as_json(conn, "run_steps")?,
        settings: fetch_table_as_json(conn, "settings")?,
        grants: fetch_table_as_json(conn, "grants")?,
        audit_queue: fetch_table_as_json(conn, "audit_queue")?,
    };

    let file = File::create(path).map_err(|e| StoreError::Migration(e.to_string()))?;
    serde_json::to_writer_pretty(file, &data).map_err(|e| StoreError::Migration(e.to_string()))?;

    Ok(())
}

fn fetch_table_as_json(
    conn: &Connection,
    table: &str,
) -> Result<Vec<serde_json::Value>, StoreError> {
    let sql = format!("SELECT * FROM {table}");
    let mut stmt = conn.prepare(&sql)?;
    let column_names: Vec<String> = stmt.column_names().into_iter().map(str::to_string).collect();

    let mut rows = stmt.query([])?;
    let mut result = Vec::new();

    while let Some(row) = rows.next()? {
        let mut map = serde_json::Map::new();
        for (i, column_name) in column_names.iter().enumerate() {
            let val = match row.get_ref(i)? {
                rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                rusqlite::types::ValueRef::Integer(v) => serde_json::json!(v),
                rusqlite::types::ValueRef::Real(v) => serde_json::json!(v),
                rusqlite::types::ValueRef::Text(v) => {
                    let s = std::str::from_utf8(v).unwrap_or("");
                    // Try to parse as JSON if it looks like it (crude heuristic, but often true for payload/value columns)
                    if s.starts_with('{') || s.starts_with('[') {
                        if let Ok(j) = serde_json::from_str(s) {
                            j
                        } else {
                            serde_json::Value::String(s.to_string())
                        }
                    } else {
                        serde_json::Value::String(s.to_string())
                    }
                }
                rusqlite::types::ValueRef::Blob(v) => serde_json::Value::String(hex::encode(v)),
            };
            map.insert(column_name.clone(), val);
        }
        result.push(serde_json::Value::Object(map));
    }

    Ok(result)
}

/// # Errors
///
/// Returns [`StoreError`] if the JSON file cannot be read/parsed, or if any
/// table insert fails.
pub fn import_from_json(conn: &mut Connection, path: &Path) -> Result<(), StoreError> {
    let file = File::open(path).map_err(|e| StoreError::Migration(e.to_string()))?;
    let data: ExportData =
        serde_json::from_reader(file).map_err(|e| StoreError::Migration(e.to_string()))?;

    let tx = conn.transaction()?;

    // Disable foreign keys during import
    tx.execute("PRAGMA foreign_keys = OFF", [])?;

    insert_table_json(&tx, "workspaces", data.workspaces)?;
    insert_table_json(&tx, "conversations", data.conversations)?;
    insert_table_json(&tx, "messages", data.messages)?;
    insert_table_json(&tx, "runs", data.runs)?;
    insert_table_json(&tx, "run_steps", data.run_steps)?;
    insert_table_json(&tx, "settings", data.settings)?;
    insert_table_json(&tx, "grants", data.grants)?;
    insert_table_json(&tx, "audit_queue", data.audit_queue)?;

    tx.execute("PRAGMA foreign_keys = ON", [])?;
    tx.commit()?;

    Ok(())
}

fn insert_table_json(
    tx: &rusqlite::Transaction,
    table: &str,
    items: Vec<serde_json::Value>,
) -> Result<(), StoreError> {
    if items.is_empty() {
        return Ok(());
    }

    for item in items {
        if let serde_json::Value::Object(map) = item {
            let cols: Vec<&String> = map.keys().collect();
            let mut vals: Vec<rusqlite::types::Value> = Vec::new();
            for k in &cols {
                let v = &map[*k];
                vals.push(json_to_rusqlite(v));
            }

            let col_names = cols.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            let placeholders = cols.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

            let sql =
                format!("INSERT OR REPLACE INTO {table} ({col_names}) VALUES ({placeholders})");
            tx.execute(&sql, rusqlite::params_from_iter(vals))?;
        }
    }
    Ok(())
}

fn json_to_rusqlite(val: &serde_json::Value) -> rusqlite::types::Value {
    match val {
        serde_json::Value::Null => rusqlite::types::Value::Null,
        serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(i64::from(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Null
            }
        }
        serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            rusqlite::types::Value::Text(val.to_string())
        }
    }
}
