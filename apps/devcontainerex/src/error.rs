use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DevContainerExError {
    Workspace {
        path: PathBuf,
        source: io::Error,
    },
    CurrentDir(io::Error),
    DockerSpawn(io::Error),
    DockerFailed {
        command: &'static str,
        stderr: String,
    },
    DockerParse(serde_json::Error),
    ContainerNotFound {
        workspace: PathBuf,
    },
    AmbiguousContainers(Vec<(String, String)>),
    CliNotFound,
}

impl fmt::Display for DevContainerExError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Workspace { path, source } => write!(
                f,
                "failed to resolve workspace path {}: {source}",
                path.display()
            ),
            Self::CurrentDir(source) => {
                write!(f, "failed to get the current directory: {source}")
            }
            Self::DockerSpawn(source) => {
                write!(f, "failed to run docker: {source}")
            }
            Self::DockerFailed { command, stderr } => {
                write!(f, "{command} failed.")?;
                let stderr = stderr.trim_end();
                if !stderr.is_empty() {
                    write!(f, "\n{stderr}")?;
                }
                Ok(())
            }
            Self::DockerParse(source) => {
                write!(f, "failed to parse docker inspect output: {source}")
            }
            Self::ContainerNotFound { workspace } => write!(
                f,
                "no matching dev container found.\n\
                 Start one first:\n\
                 \x20 devcontainer up --workspace-folder {}",
                workspace.display()
            ),
            Self::AmbiguousContainers(candidates) => {
                write!(
                    f,
                    "multiple candidate containers found; cannot determine which one to enter."
                )?;
                for (name, config_file) in candidates {
                    write!(f, "\n  {name}  ({config_file})")?;
                }
                Ok(())
            }
            Self::CliNotFound => write!(
                f,
                "devcontainer CLI not found.\n\
                 Set the DEVCONTAINEREX_DEVCONTAINER_BIN environment variable to the path of the executable.\n\
                 Example: export DEVCONTAINEREX_DEVCONTAINER_BIN=/path/to/devcontainer"
            ),
        }
    }
}

impl std::error::Error for DevContainerExError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Workspace { source, .. }
            | Self::CurrentDir(source)
            | Self::DockerSpawn(source) => Some(source),
            Self::DockerParse(source) => Some(source),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for DevContainerExError {
    fn from(source: serde_json::Error) -> Self {
        Self::DockerParse(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn docker_failed_appends_trimmed_stderr() {
        let e = DevContainerExError::DockerFailed {
            command: "docker ps",
            stderr: "boom\n".to_string(),
        };
        assert_eq!(e.to_string(), "docker ps failed.\nboom");

        let e = DevContainerExError::DockerFailed {
            command: "docker ps",
            stderr: String::new(),
        };
        assert_eq!(e.to_string(), "docker ps failed.");
    }

    #[test]
    fn ambiguous_containers_lists_candidates() {
        let e = DevContainerExError::AmbiguousContainers(vec![
            (
                "svc-api".to_string(),
                "/ws/.devcontainer/api/devcontainer.json".to_string(),
            ),
            (
                "svc-web".to_string(),
                "/ws/.devcontainer/web/devcontainer.json".to_string(),
            ),
        ]);
        let s = e.to_string();
        assert!(s.contains("svc-api  (/ws/.devcontainer/api/devcontainer.json)"));
        assert!(s.contains("svc-web  (/ws/.devcontainer/web/devcontainer.json)"));
    }

    #[test]
    fn source_is_exposed() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let e = DevContainerExError::DockerSpawn(io_err);
        assert!(e.source().is_some());
        assert!(DevContainerExError::CliNotFound.source().is_none());
    }

    #[test]
    fn question_mark_converts_to_boxed_error() {
        fn inner() -> Result<(), DevContainerExError> {
            Err(DevContainerExError::CliNotFound)
        }
        fn outer() -> Result<(), Box<dyn Error>> {
            inner()?;
            Ok(())
        }
        assert!(outer().is_err());
    }
}
