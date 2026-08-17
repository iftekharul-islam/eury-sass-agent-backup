//! `list_dir` and `glob` must route their traversal root through
//! `PathGuard`, not a non-canonicalizing `starts_with` prefix check — the
//! latter is satisfied by `..` segments and symlinks that actually resolve
//! outside the workspace.

use agent_sandbox::path::PathGuard;
use agent_sandbox::workspace::{TrustState, Workspace};
use agent_tools::Tool;
use agent_tools::fs::search::{GlobTool, ListDirTool};
use serde_json::json;
use tokio::fs;
use uuid::Uuid;

struct Fixture {
    workspace: Workspace,
    root: std::path::PathBuf,
    outside: std::path::PathBuf,
}

/// Builds `<tmp>/<id>/workspace` (the root) alongside `<tmp>/<id>/outside`,
/// so "escape the workspace" has somewhere real to escape *to*. In-workspace
/// files are written through `PathGuard`, the sanctioned write path.
async fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    let base = std::env::temp_dir().join(format!("agent_search_test_{}", Uuid::now_v7()));
    let root = base.join("workspace");
    let outside = base.join("outside");
    fs::create_dir_all(root.join("sub")).await?;
    fs::create_dir_all(&outside).await?;

    let workspace =
        Workspace { id: Uuid::now_v7(), root_path: root.clone(), trust_state: TrustState::Trusted };
    PathGuard::open_write(&workspace, std::path::Path::new("inside.txt"), b"in")?;
    PathGuard::open_write(&workspace, std::path::Path::new("sub/nested.txt"), b"nested")?;

    Ok(Fixture { workspace, root, outside })
}

#[tokio::test]
async fn list_dir_lists_workspace_contents() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;
    let result = ListDirTool.execute(json!({"path": "."}), Some(&f.workspace)).await?;

    let files = result["files"].as_array().ok_or("files should be an array")?;
    let names: Vec<&str> = files.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"inside.txt"), "expected inside.txt in {names:?}");

    let _ = fs::remove_dir_all(f.root.parent().unwrap_or(&f.root)).await;
    Ok(())
}

#[tokio::test]
async fn list_dir_rejects_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;

    // `../outside` joins to a real directory outside the root. A plain
    // `starts_with` check on the *unresolved* join passes here, which is
    // exactly the bug: only canonicalization catches it.
    let result = ListDirTool.execute(json!({"path": "../outside"}), Some(&f.workspace)).await;

    assert!(result.is_err(), "traversal outside the workspace root must be refused");

    let _ = fs::remove_dir_all(f.root.parent().unwrap_or(&f.root)).await;
    Ok(())
}

#[tokio::test]
async fn glob_rejects_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;

    let result = GlobTool
        .execute(json!({"pattern": "*.txt", "dir": "../outside"}), Some(&f.workspace))
        .await;

    assert!(result.is_err(), "traversal outside the workspace root must be refused");

    let _ = fs::remove_dir_all(f.root.parent().unwrap_or(&f.root)).await;
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn list_dir_rejects_symlink_escape() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;

    // A symlink *inside* the workspace pointing outside it. The unresolved
    // path is inside the root, so only canonicalization catches this.
    let link = f.root.join("escape");
    std::os::unix::fs::symlink(&f.outside, &link)?;

    let result = ListDirTool.execute(json!({"path": "escape"}), Some(&f.workspace)).await;

    assert!(result.is_err(), "a symlink resolving outside the workspace must be refused");

    let _ = fs::remove_dir_all(f.root.parent().unwrap_or(&f.root)).await;
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn glob_rejects_symlink_escape() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;

    let link = f.root.join("escape");
    std::os::unix::fs::symlink(&f.outside, &link)?;

    let result =
        GlobTool.execute(json!({"pattern": "*.txt", "dir": "escape"}), Some(&f.workspace)).await;

    assert!(result.is_err(), "a symlink resolving outside the workspace must be refused");

    let _ = fs::remove_dir_all(f.root.parent().unwrap_or(&f.root)).await;
    Ok(())
}

#[tokio::test]
async fn glob_matches_within_the_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;

    let result =
        GlobTool.execute(json!({"pattern": "**/*.txt", "dir": "."}), Some(&f.workspace)).await?;

    let matches = result["matches"].as_array().ok_or("matches should be an array")?;
    let names: Vec<&str> = matches.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"inside.txt"), "expected inside.txt in {names:?}");

    let _ = fs::remove_dir_all(f.root.parent().unwrap_or(&f.root)).await;
    Ok(())
}
