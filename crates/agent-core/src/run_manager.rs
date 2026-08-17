use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::engine::AgentEngine;
use agent_types::errors::AgentError;
use agent_types::events::AgentEvent;
use agent_types::requests::{RunOutcome, RunRequest};

pub struct RunManager {
    engine: Arc<dyn AgentEngine>,
    active_runs: Arc<Mutex<HashMap<Uuid, CancellationToken>>>,
}

impl RunManager {
    pub fn new(engine: Arc<dyn AgentEngine>) -> Self {
        Self { engine, active_runs: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// # Errors
    ///
    /// Returns an error if a foreground run is already active for this conversation,
    /// or if the underlying engine's run fails.
    pub async fn start_run(
        &self,
        request: RunRequest,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Result<RunOutcome, AgentError> {
        let conv_id = request.conversation_id;
        let run_id = request.run_id;

        let cancel_token = CancellationToken::new();

        {
            let mut runs = self.active_runs.lock().await;
            if runs.contains_key(&conv_id) {
                return Err(AgentError::InvalidTransition(
                    "A foreground run is already active for this conversation".into(),
                ));
            }
            runs.insert(conv_id, cancel_token.clone());
        }

        // Notify UI about start
        let _ = tx.send(AgentEvent::Meta { run_id, status: "queued".to_string() }).await;

        // Clone engine to move it to a task if needed, but here we just await it directly
        // because we are already in an async function. The caller should spawn this if they
        // want it to run in the background.
        let result = self.engine.run_stream(request, tx, cancel_token).await;

        {
            let mut runs = self.active_runs.lock().await;
            runs.remove(&conv_id);
        }

        result
    }

    /// # Errors
    ///
    /// Returns an error if no active run exists for the given conversation.
    ///
    /// Frees the conversation's slot immediately rather than waiting for the
    /// cancelled run to unwind and unregister itself. Without this, a user
    /// hitting Stop and immediately sending a new message can hit "a
    /// foreground run is already active" while the old run is still winding
    /// down (the desktop AI SDK path cancels via its own `AbortController`,
    /// not this token, so its cleanup is async and not guaranteed to have
    /// finished by the time the next run tries to register).
    pub async fn cancel_run(&self, conversation_id: Uuid) -> Result<(), AgentError> {
        let mut runs = self.active_runs.lock().await;
        if let Some(token) = runs.remove(&conversation_id) {
            token.cancel();
            Ok(())
        } else {
            Err(AgentError::InvalidTransition("No active run found for conversation".into()))
        }
    }

    /// Registers a foreground run for cancellation without starting the Rust inference loop.
    ///
    /// Used by the desktop AI SDK path where TypeScript owns model streaming.
    pub async fn register_run(&self, conversation_id: Uuid) -> Result<CancellationToken, AgentError> {
        let mut runs = self.active_runs.lock().await;
        if runs.contains_key(&conversation_id) {
            return Err(AgentError::InvalidTransition(
                "A foreground run is already active for this conversation".into(),
            ));
        }
        let token = CancellationToken::new();
        runs.insert(conversation_id, token.clone());
        Ok(token)
    }

    /// Clears a run registered via [`Self::register_run`].
    pub async fn unregister_run(&self, conversation_id: Uuid) {
        let mut runs = self.active_runs.lock().await;
        runs.remove(&conversation_id);
    }

    /// Returns the cancellation token for an active run, if any.
    pub async fn cancel_token_for(&self, conversation_id: Uuid) -> Option<CancellationToken> {
        let runs = self.active_runs.lock().await;
        runs.get(&conversation_id).cloned()
    }
}
