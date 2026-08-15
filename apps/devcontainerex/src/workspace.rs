//! Workspace path resolution.

use crate::error::DevContainerExError;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Determines the absolute workspace path.
///
/// Resolution order:
/// 1. `flag_value` (the value peeked from `-w` / `--workspace-folder`)
/// 2. `git rev-parse --show-toplevel`
/// 3. the current directory
///
/// The result is canonicalized; returns an error if the path cannot be
/// resolved.
pub fn determine(flag_value: Option<&str>) -> Result<PathBuf, DevContainerExError> {
    let raw = if let Some(p) = flag_value {
        PathBuf::from(p)
    } else if let Some(top) = git_toplevel() {
        top
    } else {
        std::env::current_dir().map_err(DevContainerExError::CurrentDir)?
    };

    raw.canonicalize()
        .map_err(|source| DevContainerExError::Workspace { path: raw, source })
}

/// Returns the repository root via `git rev-parse --show-toplevel`,
/// or `None` when outside a git repository (or git is unavailable).
fn git_toplevel() -> Option<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}
