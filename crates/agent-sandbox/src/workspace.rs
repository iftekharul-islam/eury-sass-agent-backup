use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustState {
    Trusted,
    Untrusted,
}

/// Tracks which workspace roots the user has explicitly trusted.
///
/// Trust is keyed by **canonical absolute path** and defaults to
/// [`TrustState::Untrusted`] — a folder the user has never answered the trust
/// prompt for is read-only, never implicitly trusted. Revoking trust also
/// clears it back to untrusted (per the trust-state table: `revoked` returns
/// to untrusted).
///
/// This store is in-memory today, so trust does not survive an app restart
/// (the user is re-prompted). Persisting it belongs with the `agent-store`
/// wiring that owns the `workspaces` table.
#[derive(Debug, Default)]
pub struct TrustStore {
    trusted_paths: Mutex<HashSet<PathBuf>>,
}

impl TrustStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Canonicalizes `path` so trust can't be bypassed by a differently
    /// spelled route to the same directory (`..` segments, symlinks, or a
    /// trailing slash). Falls back to the path as given when it can't be
    /// canonicalized (e.g. it no longer exists), which is the conservative
    /// choice: an unmatched key reads as untrusted.
    fn key(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }

    #[must_use]
    pub fn trust_state(&self, path: &Path) -> TrustState {
        let key = Self::key(path);
        let trusted = self.trusted_paths.lock().unwrap_or_else(PoisonError::into_inner);
        if trusted.contains(&key) { TrustState::Trusted } else { TrustState::Untrusted }
    }

    #[must_use]
    pub fn is_trusted(&self, path: &Path) -> bool {
        self.trust_state(path) == TrustState::Trusted
    }

    /// Sets trust for `path`. Passing `false` revokes it back to untrusted.
    pub fn set_trust(&self, path: &Path, trusted: bool) {
        let key = Self::key(path);
        let mut paths = self.trusted_paths.lock().unwrap_or_else(PoisonError::into_inner);
        if trusted {
            paths.insert(key);
        } else {
            paths.remove(&key);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Uuid,
    pub root_path: PathBuf,
    pub trust_state: TrustState,
}

impl Workspace {
    #[must_use]
    pub fn is_trusted(&self) -> bool {
        self.trust_state == TrustState::Trusted
    }
}

pub struct WorkspaceRegistry {
    workspaces: RwLock<HashMap<Uuid, Arc<Workspace>>>,
}

impl Default for WorkspaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self { workspaces: RwLock::new(HashMap::new()) }
    }

    pub async fn register(
        &self,
        id: Uuid,
        root_path: PathBuf,
        trust_state: TrustState,
    ) -> Arc<Workspace> {
        let ws = Arc::new(Workspace { id, root_path, trust_state });
        let mut map = self.workspaces.write().await;
        map.insert(id, ws.clone());
        ws
    }

    pub async fn get(&self, id: &Uuid) -> Option<Arc<Workspace>> {
        let map = self.workspaces.read().await;
        map.get(id).cloned()
    }

    pub async fn remove(&self, id: &Uuid) {
        let mut map = self.workspaces.write().await;
        map.remove(id);
    }

    pub async fn update_trust(&self, id: &Uuid, trust_state: TrustState) -> Option<Arc<Workspace>> {
        let mut map = self.workspaces.write().await;
        if let Some(ws) = map.get(id) {
            let updated =
                Arc::new(Workspace { id: ws.id, root_path: ws.root_path.clone(), trust_state });
            map.insert(*id, updated.clone());
            Some(updated)
        } else {
            None
        }
    }
}
