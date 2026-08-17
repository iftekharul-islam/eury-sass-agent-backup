//! Exercises the encrypted store against a real `SQLCipher` database file,
//! including the migration chain and the read path the desktop app uses.

use agent_store::actor::{StoreActorHandle, spawn_actor_with_key};
use rusqlite::types::Value;
use std::error::Error;

/// A fixed `SQLCipher` key for tests.
///
/// The keychain-backed path can't be used here: CI runners have no keychain
/// daemon, and macOS gates keychain access behind an interactive prompt for
/// each freshly built, unsigned test binary. Supplying the key directly keeps
/// these tests hermetic while still exercising the real encrypted-database
/// code path.
const TEST_KEY: &str = "0123456789abcdef0123456789abcdef";

fn open(path: &str) -> Result<StoreActorHandle, Box<dyn Error>> {
    Ok(spawn_actor_with_key(path, TEST_KEY)?)
}

/// Each test gets its own database file so they can run in parallel without
/// sharing migration state.
fn temp_db_path(label: &str) -> String {
    std::env::temp_dir()
        .join(format!("eury-store-test-{label}-{}.db", uuid::Uuid::now_v7()))
        .to_string_lossy()
        .to_string()
}

/// Removes the temp database and its pre-migration backups.
///
/// `PathGuard` exists to contain agent/tool access to *workspace* content;
/// these are this test's own temp files, so the workspace-containment rule
/// doesn't apply — the same reasoning that exempts `agent-store` itself in
/// `scripts/check-boundaries.mjs`.
#[allow(clippy::disallowed_methods)]
fn cleanup(path: &str) {
    let _ = std::fs::remove_file(path);
    for suffix in ["db.pre-1", "db.pre-2"] {
        let _ = std::fs::remove_file(std::path::Path::new(path).with_extension(suffix));
    }
}

#[tokio::test]
async fn migrations_apply_and_are_recorded() -> Result<(), Box<dyn Error>> {
    let path = temp_db_path("migrations");
    let store = open(&path)?;

    let rows = store
        .query("SELECT version FROM schema_migrations ORDER BY version".to_string(), vec![])
        .await?;
    let versions: Vec<i64> = rows
        .iter()
        .filter_map(|r| r.first())
        .filter_map(|v| match v {
            Value::Integer(i) => Some(*i),
            _ => None,
        })
        .collect();
    assert_eq!(versions, vec![1, 2], "both migrations must be applied and recorded");

    store.shutdown().await;
    cleanup(&path);
    Ok(())
}

#[tokio::test]
async fn settings_round_trip_through_the_encrypted_store() -> Result<(), Box<dyn Error>> {
    let path = temp_db_path("settings");
    let store = open(&path)?;

    store
        .execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value"
                .to_string(),
            vec![
                Value::Text("app_settings".into()),
                Value::Text(r#"{"theme":"light","accent":"ember","density":"compact"}"#.into()),
            ],
        )
        .await?;

    let rows = store
        .query(
            "SELECT value FROM settings WHERE key = ?1".to_string(),
            vec![Value::Text("app_settings".into())],
        )
        .await?;

    let Some(Value::Text(json)) = rows.first().and_then(|r| r.first()) else {
        return Err("expected a settings row".into());
    };
    assert!(json.contains("\"theme\":\"light\""));

    store.shutdown().await;
    cleanup(&path);
    Ok(())
}

#[tokio::test]
async fn workspace_trust_persists_and_defaults_to_untrusted() -> Result<(), Box<dyn Error>> {
    let path = temp_db_path("trust");
    let store = open(&path)?;

    // A workspace inserted without an explicit trust state must default to
    // untrusted — "unknown" reads as read-only, never as trusted.
    store
        .execute(
            "INSERT INTO workspaces (id, path, name) VALUES (?1, ?2, ?3)".to_string(),
            vec![
                Value::Text("ws-1".into()),
                Value::Text("/tmp/ws-default".into()),
                Value::Text("ws-default".into()),
            ],
        )
        .await?;
    let rows = store
        .query(
            "SELECT trust_state FROM workspaces WHERE path = ?1".to_string(),
            vec![Value::Text("/tmp/ws-default".into())],
        )
        .await?;
    assert_eq!(rows.first().and_then(|r| r.first()), Some(&Value::Text("untrusted".into())));

    // The upsert the trust command issues flips it to trusted and is
    // idempotent on repeat.
    for _ in 0..2 {
        store
            .execute(
                "INSERT INTO workspaces (id, path, name, trust_state) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(path) DO UPDATE SET trust_state = excluded.trust_state"
                    .to_string(),
                vec![
                    Value::Text("ws-2".into()),
                    Value::Text("/tmp/ws-trusted".into()),
                    Value::Text("ws-trusted".into()),
                    Value::Text("trusted".into()),
                ],
            )
            .await?;
    }

    let trusted = store
        .query("SELECT path FROM workspaces WHERE trust_state = 'trusted'".to_string(), vec![])
        .await?;
    assert_eq!(trusted.len(), 1, "the upsert must not duplicate rows");
    assert_eq!(trusted[0].first(), Some(&Value::Text("/tmp/ws-trusted".into())));

    store.shutdown().await;
    cleanup(&path);
    Ok(())
}

#[tokio::test]
async fn data_survives_reopening_the_database() -> Result<(), Box<dyn Error>> {
    let path = temp_db_path("reopen");

    {
        let store = open(&path)?;
        store
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)".to_string(),
                vec![Value::Text("k".into()), Value::Text("\"v\"".into())],
            )
            .await?;
        store.shutdown().await;
    }

    // Reopening decrypts with the same key and finds the row —
    // this is what makes persistence real rather than per-process.
    let store = open(&path)?;
    let rows = store
        .query(
            "SELECT value FROM settings WHERE key = ?1".to_string(),
            vec![Value::Text("k".into())],
        )
        .await?;
    assert_eq!(rows.first().and_then(|r| r.first()), Some(&Value::Text("\"v\"".into())));

    store.shutdown().await;
    cleanup(&path);
    Ok(())
}

/// Debug builds must keep the `SQLCipher` key in a `0600` sidecar file rather
/// than the OS keychain: keychain ACLs are bound to a binary's code signature,
/// so every rebuild of an ad-hoc-signed dev app re-prompts for the login
/// password. `open_or_create` is the path the desktop app takes.
#[cfg(debug_assertions)]
#[test]
#[allow(clippy::disallowed_methods)]
fn debug_builds_key_the_database_from_a_sidecar_file() -> Result<(), Box<dyn Error>> {
    // SAFETY guard: an explicit key would short-circuit the path under test.
    if std::env::var("EURY_AGENT_DB_KEY").is_ok() {
        return Ok(());
    }

    let path = temp_db_path("sidecar-key");
    let key_path = format!("{path}.key");

    {
        let store = agent_store::db::Store::open_or_create(&path)?;
        drop(store);
    }

    let key = std::fs::read_to_string(&key_path)?;
    assert_eq!(key.trim().len(), 64, "expected a 32-byte hex key");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&key_path)?.permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "the key file must be owner-only");
    }

    // Reopening reuses the same key, so the database stays readable instead of
    // being quarantined as corrupt on every launch.
    {
        let store = agent_store::db::Store::open_or_create(&path)?;
        drop(store);
    }
    assert_eq!(std::fs::read_to_string(&key_path)?, key);

    cleanup(&path);
    let _ = std::fs::remove_file(&key_path);
    Ok(())
}
