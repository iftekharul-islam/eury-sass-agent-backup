use crate::workspace::Workspace;
use rand::RngExt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathGuardError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Path escaped workspace root")]
    EscapeAttempt,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Workspace is untrusted and operation requires write access")]
    UntrustedWorkspaceWrite,
}

pub struct PathGuard;

impl PathGuard {
    /// Opens a file for reading after verifying it is within the workspace root.
    ///
    /// # Errors
    ///
    /// Returns [`PathGuardError::EscapeAttempt`] if `rel_path` resolves outside
    /// the workspace root, or [`PathGuardError::Io`]/[`PathGuardError::InvalidPath`]
    /// if the path cannot be resolved or opened.
    pub fn open_read(workspace: &Workspace, rel_path: &Path) -> Result<File, PathGuardError> {
        let abs_path = Self::resolve_and_verify(workspace, rel_path)?;

        // Open without following final symlink if possible, or just standard open
        // In a real implementation we would open parent dir handle and openat.
        // For phase 5, we do a basic canonicalization check.
        let f = File::open(&abs_path)?;
        Ok(f)
    }

    /// Opens a file for atomic writing (writes to temp file, then renames)
    ///
    /// # Errors
    ///
    /// Returns [`PathGuardError::UntrustedWorkspaceWrite`] if the workspace is
    /// not trusted, [`PathGuardError::EscapeAttempt`] if `rel_path` resolves
    /// outside the workspace root, or [`PathGuardError::Io`] on write failure.
    pub fn open_write(
        workspace: &Workspace,
        rel_path: &Path,
        content: &[u8],
    ) -> Result<(), PathGuardError> {
        if !workspace.is_trusted() {
            return Err(PathGuardError::UntrustedWorkspaceWrite);
        }

        let abs_path = Self::resolve_and_verify(workspace, rel_path)?;
        let parent = abs_path
            .parent()
            .ok_or_else(|| PathGuardError::InvalidPath("No parent directory".into()))?;

        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        // Generate temp file name in the same directory
        let mut rng = rand::rng();
        let rng_val = rng.random::<u32>();
        let temp_name = format!(".tmp.{rng_val}.write");
        let temp_path = parent.join(temp_name);

        let mut f = OpenOptions::new().write(true).create_new(true).open(&temp_path)?;
        f.write_all(content)?;
        f.sync_all()?; // fsync

        // Atomic rename
        std::fs::rename(&temp_path, &abs_path).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            PathGuardError::Io(e)
        })?;

        Ok(())
    }

    /// Resolves a directory to walk, verifying it stays inside the workspace
    /// root, and returns `(canonical_dir, canonical_root)`.
    ///
    /// Directory-listing tools (`list_dir`, `glob`) must route their traversal
    /// root through here rather than doing a plain `starts_with` prefix check:
    /// a non-canonicalizing check is satisfied by `..` segments and symlinks
    /// that actually resolve outside the workspace. The canonical root is
    /// returned alongside so callers strip results against the same resolved
    /// prefix they were verified against.
    ///
    /// # Errors
    ///
    /// Returns [`PathGuardError::EscapeAttempt`] if `rel_path` resolves outside
    /// the workspace root, [`PathGuardError::InvalidPath`] if it is not a
    /// directory, or [`PathGuardError::Io`] if it cannot be resolved.
    pub fn resolve_dir(
        workspace: &Workspace,
        rel_path: &Path,
    ) -> Result<(PathBuf, PathBuf), PathGuardError> {
        let resolved = Self::resolve_and_verify(workspace, rel_path)?;
        if !resolved.is_dir() {
            return Err(PathGuardError::InvalidPath("Not a directory".into()));
        }
        let canon_root = std::fs::canonicalize(&workspace.root_path)?;
        Ok((resolved, canon_root))
    }

    fn resolve_and_verify(
        workspace: &Workspace,
        rel_path: &Path,
    ) -> Result<PathBuf, PathGuardError> {
        let path_str = rel_path.to_string_lossy();
        if path_str.contains('\0') {
            return Err(PathGuardError::InvalidPath("Path contains NUL byte".into()));
        }

        let full_path = workspace.root_path.join(rel_path);

        // We canonicalize the path. If it exists, canonicalize resolves symlinks.
        // If it does not exist, we canonicalize the parent.
        let resolved = if full_path.exists() {
            std::fs::canonicalize(&full_path)?
        } else {
            let parent = full_path
                .parent()
                .ok_or_else(|| PathGuardError::InvalidPath("No parent".into()))?;
            if parent.exists() {
                let file_name = full_path
                    .file_name()
                    .ok_or_else(|| PathGuardError::InvalidPath("No file name".into()))?;
                let mut can = std::fs::canonicalize(parent)?;
                can.push(file_name);
                can
            } else {
                // Return original full_path for now if we can't canonicalize parent, or fail.
                // For simplicity, we just check if it starts with root.
                full_path.clone()
            }
        };

        let canon_root = std::fs::canonicalize(&workspace.root_path)?;

        if !resolved.starts_with(&canon_root) {
            return Err(PathGuardError::EscapeAttempt);
        }

        Ok(resolved)
    }
}
