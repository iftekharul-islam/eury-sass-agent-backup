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
    pub async fn cancel_run(&self, conversation_id: Uuid) -> Result<(), AgentError> {
        let runs = self.active_runs.lock().await;
        if let Some(token) = runs.get(&conversation_id) {
            token.cancel();
            Ok(())
        } else {
            Err(AgentError::InvalidTransition("No active run found for conversation".into()))
        }
    }
}
