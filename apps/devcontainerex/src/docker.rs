use crate::error::DevContainerExError;
use std::process::Command;

pub struct Container {
    pub name: String,
    pub config_file: String,
    pub local_folder: Option<String>,
}

pub fn running_containers() -> Result<Vec<Container>, DevContainerExError> {
    let out = Command::new("docker")
        .args(["ps", "-q"])
        .output()
        .map_err(DevContainerExError::DockerSpawn)?;
    if !out.status.success() {
        return Err(DevContainerExError::DockerFailed {
            command: "docker ps",
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let ids: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let out = Command::new("docker")
        .arg("inspect")
        .args(&ids)
        .output()
        .map_err(DevContainerExError::DockerSpawn)?;
    if !out.status.success() {
        return Err(DevContainerExError::DockerFailed {
            command: "docker inspect",
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    let json: serde_json::Value = serde_json::from_slice(&out.stdout)?;

    let mut containers = Vec::new();
    for item in json.as_array().into_iter().flatten() {
        let labels = &item["Config"]["Labels"];
        let Some(config_file) = labels["devcontainer.config_file"].as_str() else {
            continue;
        };
        let name = item["Name"]
            .as_str()
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();
        containers.push(Container {
            name,
            config_file: config_file.to_string(),
            local_folder: labels["devcontainer.local_folder"]
                .as_str()
                .map(str::to_string),
        });
    }
    Ok(containers)
}
