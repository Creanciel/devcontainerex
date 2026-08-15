use crate::error::DevContainerExError;
use std::path::PathBuf;

pub fn find_devcontainer_bin() -> Result<PathBuf, DevContainerExError> {
    if let Some(bin) = std::env::var_os("DEVCONTAINEREX_DEVCONTAINER_BIN") {
        return Ok(PathBuf::from(bin));
    }
    if let Some(bin) = from_path() {
        return Ok(bin);
    }
    Err(DevContainerExError::CliNotFound)
}

fn from_path() -> Option<PathBuf> {
    let self_exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok());
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join("devcontainer");
        if !cand.is_file() {
            continue;
        }
        let Ok(canon) = cand.canonicalize() else {
            continue;
        };
        if Some(&canon) == self_exe.as_ref() {
            continue;
        }
        return Some(cand);
    }
    None
}
