use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub image: String,
    pub status: String,
    pub names: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerEnvironment {
    pub dockerfile_path: Option<PathBuf>,
    pub compose_file_path: Option<PathBuf>,
    pub is_docker_available: bool,
    pub active_containers: Vec<DockerContainer>,
}

pub struct DockerManager;

impl DockerManager {
    /// Detect Dockerfiles and docker-compose files in a project root
    pub fn inspect_project<P: AsRef<Path>>(root: P) -> DockerEnvironment {
        let root = root.as_ref();
        let dockerfile = root.join("Dockerfile");
        let compose1 = root.join("docker-compose.yml");
        let compose2 = root.join("compose.yaml");

        let dockerfile_path = if dockerfile.exists() { Some(dockerfile) } else { None };
        let compose_file_path = if compose1.exists() {
            Some(compose1)
        } else if compose2.exists() {
            Some(compose2)
        } else {
            None
        };

        let is_docker_available = Self::check_docker_available();
        let active_containers = if is_docker_available {
            Self::list_containers().unwrap_or_default()
        } else {
            Vec::new()
        };

        DockerEnvironment {
            dockerfile_path,
            compose_file_path,
            is_docker_available,
            active_containers,
        }
    }

    pub fn check_docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn list_containers() -> Result<Vec<DockerContainer>, String> {
        let output = std::process::Command::new("docker")
            .args(["ps", "--format", "{{.ID}}\t{{.Image}}\t{{.Status}}\t{{.Names}}"])
            .output()
            .map_err(|e| format!("Failed to run docker ps: {}", e))?;

        if !output.status.success() {
            return Err("Docker daemon not running or access denied".to_string());
        }

        let mut containers = Vec::new();
        let out_str = String::from_utf8_lossy(&output.stdout);
        for line in out_str.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                containers.push(DockerContainer {
                    id: parts[0].to_string(),
                    image: parts[1].to_string(),
                    status: parts[2].to_string(),
                    names: parts[3].to_string(),
                });
            }
        }

        Ok(containers)
    }

    /// Run a command inside a Docker container
    pub fn exec_in_container(container_id: &str, command: &[&str]) -> Result<String, String> {
        let mut cmd = std::process::Command::new("docker");
        cmd.args(["exec", container_id]);
        cmd.args(command);

        let output = cmd.output().map_err(|e| format!("Failed to exec in container: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}
