use agent_sandbox::workspace::{TrustState, TrustStore};
use std::path::PathBuf;

fn temp_dir_named(suffix: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("eury-trust-test-{}-{suffix}", uuid::Uuid::now_v7()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn unknown_paths_default_to_untrusted() {
    let store = TrustStore::new();
    let dir = temp_dir_named("unknown");

    assert_eq!(store.trust_state(&dir), TrustState::Untrusted);
    assert!(!store.is_trusted(&dir));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trust_can_be_granted_and_revoked() {
    let store = TrustStore::new();
    let dir = temp_dir_named("grant");

    store.set_trust(&dir, true);
    assert_eq!(store.trust_state(&dir), TrustState::Trusted);

    // Revoking returns the workspace to untrusted, per the trust-state table.
    store.set_trust(&dir, false);
    assert_eq!(store.trust_state(&dir), TrustState::Untrusted);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trust_is_keyed_by_canonical_path() {
    let store = TrustStore::new();
    let dir = temp_dir_named("canonical");
    let nested = dir.join("nested");
    let _ = std::fs::create_dir_all(&nested);

    store.set_trust(&dir, true);

    // The same directory reached via a `..` segment must resolve to the same
    // trust entry — otherwise trust could be bypassed by respelling the path.
    let equivalent = nested.join("..");
    assert_eq!(
        store.trust_state(&equivalent),
        TrustState::Trusted,
        "a differently spelled route to a trusted directory must still be trusted"
    );

    // A genuinely different directory is unaffected.
    let other = temp_dir_named("other");
    assert_eq!(store.trust_state(&other), TrustState::Untrusted);

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&other);
}
