use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// How the user answered an approval card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalOutcome {
    Denied,
    /// Approve only this call; do not remember it for later commands.
    Once,
    /// Remember for the rest of this app session (all similar commands).
    Session,
}

pub struct ApprovalWaiter {
    pending: Mutex<HashMap<String, oneshot::Sender<ApprovalOutcome>>>,
}

impl ApprovalWaiter {
    #[must_use]
    pub fn new() -> Self {
        Self { pending: Mutex::new(HashMap::new()) }
    }

    pub fn register(&self, tool_call_id: String) -> oneshot::Receiver<ApprovalOutcome> {
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(tool_call_id, tx);
        rx
    }

    pub fn resolve(&self, tool_call_id: &str, outcome: ApprovalOutcome) -> bool {
        if let Some(tx) = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(tool_call_id)
        {
            let _ = tx.send(outcome);
            true
        } else {
            false
        }
    }
}

impl Default for ApprovalWaiter {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedApprovalWaiter = Arc<ApprovalWaiter>;
