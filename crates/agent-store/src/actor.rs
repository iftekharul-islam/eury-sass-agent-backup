use crate::db::{Store, StoreError};
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};

pub enum StoreCommand {
    Execute {
        sql: String,
        params: Vec<rusqlite::types::Value>,
        reply: oneshot::Sender<Result<usize, StoreError>>,
    },
    Query {
        sql: String,
        params: Vec<rusqlite::types::Value>,
        reply: oneshot::Sender<Result<Vec<Vec<rusqlite::types::Value>>, StoreError>>,
    },
    Shutdown,
}

pub struct StoreActorHandle {
    sender: mpsc::Sender<StoreCommand>,
}

impl StoreActorHandle {
    /// # Errors
    ///
    /// Returns [`StoreError::Migration`] if the actor thread has shut down,
    /// or any error the underlying `execute` call produced.
    pub async fn execute(
        &self,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<usize, StoreError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(StoreCommand::Execute { sql, params, reply: tx })
            .await
            .map_err(|_| StoreError::Migration("Actor closed".into()))?;
        rx.await.map_err(|_| StoreError::Migration("Actor closed".into()))?
    }

    /// Runs a read query, returning positional row values.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Migration`] if the actor thread has shut down,
    /// or any error the underlying query produced.
    pub async fn query(
        &self,
        sql: String,
        params: Vec<rusqlite::types::Value>,
    ) -> Result<Vec<Vec<rusqlite::types::Value>>, StoreError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(StoreCommand::Query { sql, params, reply: tx })
            .await
            .map_err(|_| StoreError::Migration("Actor closed".into()))?;
        rx.await.map_err(|_| StoreError::Migration("Actor closed".into()))?
    }

    pub async fn shutdown(&self) {
        let _ = self.sender.send(StoreCommand::Shutdown).await;
    }
}

pub struct StoreActor {
    store: Store,
    receiver: mpsc::Receiver<StoreCommand>,
}

impl StoreActor {
    /// # Errors
    ///
    /// Returns any error from [`Store::open_or_create`] or
    /// [`crate::migrations::run_migrations`].
    pub fn new(path: &str) -> Result<(Self, StoreActorHandle), StoreError> {
        Self::from_store(Store::open_or_create(path)?, path)
    }

    /// Like [`Self::new`], but with an explicitly supplied `SQLCipher` key —
    /// see [`Store::open_or_create_with_key`] for when that is needed.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Store::open_or_create_with_key`] or
    /// [`crate::migrations::run_migrations`].
    pub fn new_with_key(path: &str, key: &str) -> Result<(Self, StoreActorHandle), StoreError> {
        Self::from_store(Store::open_or_create_with_key(path, key)?, path)
    }

    fn from_store(mut store: Store, path: &str) -> Result<(Self, StoreActorHandle), StoreError> {
        // Run migrations synchronously on startup, on the same connection the
        // actor will go on to use — opening a second one just for migrations
        // risks the two disagreeing about the database's state.
        crate::migrations::run_migrations(&mut store.conn, &PathBuf::from(path))?;

        let (tx, rx) = mpsc::channel(100);
        let actor = Self { store, receiver: rx };
        let handle = StoreActorHandle { sender: tx };
        Ok((actor, handle))
    }

    pub fn run(mut self) {
        // Runs in a dedicated thread since SQLite is synchronous
        while let Some(cmd) = self.receiver.blocking_recv() {
            match cmd {
                StoreCommand::Execute { sql, params, reply } => {
                    let result = self.handle_execute(&sql, &params);
                    let _ = reply.send(result);
                }
                StoreCommand::Query { sql, params, reply } => {
                    let result = self.handle_query(&sql, &params);
                    let _ = reply.send(result);
                }
                StoreCommand::Shutdown => break,
            }
        }
    }

    /// # Errors
    ///
    /// Returns any error the underlying `DELETE` transaction produces.
    pub fn prune_old_runs(&mut self, days_old: i64) -> Result<usize, StoreError> {
        let sql =
            format!("DELETE FROM runs WHERE created_at < datetime('now', '-{days_old} days')");
        let tx = self.store.conn.transaction()?;
        let res = tx.execute(&sql, [])?;

        // Also cleanup old backups
        let _ = tx.execute("PRAGMA incremental_vacuum", []);

        tx.commit()?;
        Ok(res)
    }

    fn handle_execute(
        &mut self,
        sql: &str,
        params: &[rusqlite::types::Value],
    ) -> Result<usize, StoreError> {
        let tx = self.store.conn.transaction()?;
        let res = tx.execute(sql, rusqlite::params_from_iter(params.iter()))?;
        tx.commit()?;
        Ok(res)
    }

    fn handle_query(
        &self,
        sql: &str,
        params: &[rusqlite::types::Value],
    ) -> Result<Vec<Vec<rusqlite::types::Value>>, StoreError> {
        let mut stmt = self.store.conn.prepare(sql)?;
        let column_count = stmt.column_count();
        let mut rows = stmt.query(rusqlite::params_from_iter(params.iter()))?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let mut row_values = Vec::with_capacity(column_count);
            for i in 0..column_count {
                row_values.push(row.get::<_, rusqlite::types::Value>(i)?);
            }
            result.push(row_values);
        }
        Ok(result)
    }
}

/// # Errors
///
/// Returns any error from [`StoreActor::new`].
pub fn spawn_actor(path: &str) -> Result<StoreActorHandle, StoreError> {
    Ok(spawn(StoreActor::new(path)?))
}

/// Like [`spawn_actor`], but with an explicitly supplied `SQLCipher` key —
/// see [`Store::open_or_create_with_key`] for when that is needed.
///
/// # Errors
///
/// Returns any error from [`StoreActor::new_with_key`].
pub fn spawn_actor_with_key(path: &str, key: &str) -> Result<StoreActorHandle, StoreError> {
    Ok(spawn(StoreActor::new_with_key(path, key)?))
}

fn spawn((actor, handle): (StoreActor, StoreActorHandle)) -> StoreActorHandle {
    // Spawn a dedicated OS thread for SQLite to avoid blocking async workers
    std::thread::spawn(move || {
        actor.run();
    });

    handle
}
