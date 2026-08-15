mod args;
mod docker;
mod error;
mod resolver;
mod workspace;

use args::{Args, ExecArgs};
use error::DevContainerExError;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

fn main() -> ! {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<(), DevContainerExError> {
    let argv: Vec<String> = std::env::args().skip(1).collect();

    match Args::parse(argv) {
        Args::Exec(exec_args) => run_exec(exec_args),
        Args::Passthrough(argv) => passthrough(argv),
    }
}

fn run_exec(a: ExecArgs) -> Result<(), DevContainerExError> {
    if a.help_requested {
        return passthrough(a.into_argv(Vec::new()));
    }

    let ws = workspace::determine(a.workspace_folder.as_deref())?;
    let containers = docker::running_containers()?;

    let candidates: Vec<&docker::Container> = containers
        .iter()
        .filter(|c| c.local_folder.is_some() && is_under_workspace(&c.config_file, &ws))
        .collect();

    let container = match candidates.as_slice() {
        [] => return Err(DevContainerExError::ContainerNotFound { workspace: ws }),
        [one] => *one,
        many => {
            return Err(DevContainerExError::AmbiguousContainers(
                many.iter()
                    .map(|c| (c.name.clone(), c.config_file.clone()))
                    .collect(),
            ));
        }
    };

    let label = container.local_folder.as_ref().unwrap();

    let mut insert = Vec::new();
    if !a.has_id_label {
        insert.push("--id-label".to_string());
        insert.push(format!("devcontainer.local_folder={label}"));
    }
    if !a.has_workspace_flag {
        insert.push("--workspace-folder".to_string());
        insert.push(ws.display().to_string());
    }

    let bin = resolver::find_devcontainer_bin()?;
    exec(&bin, a.into_argv(insert))
}

fn passthrough(argv: Vec<String>) -> Result<(), DevContainerExError> {
    let bin = resolver::find_devcontainer_bin()?;
    exec(&bin, argv)
}

fn exec(bin: &Path, args: Vec<String>) -> ! {
    let err = Command::new(bin).args(&args).exec();
    eprintln!("failed to launch devcontainer: {err}");
    std::process::exit(127);
}

fn is_under_workspace(config_file: &str, ws: &Path) -> bool {
    let ws = ws.to_string_lossy();
    let ws = ws.trim_end_matches('/');
    config_file
        .strip_prefix(ws)
        .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn config_file_under_workspace() {
        let ws = PathBuf::from("/home/user/projects/foo");
        assert!(is_under_workspace(
            "/home/user/projects/foo/.devcontainer/devcontainer.json",
            &ws
        ));
        assert!(is_under_workspace(
            "/home/user/projects/foo/.devcontainer/api/devcontainer.json",
            &ws
        ));
        assert!(is_under_workspace(
            "/home/user/projects/foo/devcontainer.json",
            &ws
        ));
    }

    #[test]
    fn sibling_directory_is_not_matched() {
        let ws = PathBuf::from("/home/user/projects/foo");
        assert!(!is_under_workspace(
            "/home/user/projects/foobar/.devcontainer/devcontainer.json",
            &ws
        ));
        assert!(!is_under_workspace(
            "/home/user/projects/bar/.devcontainer/devcontainer.json",
            &ws
        ));
    }
}
