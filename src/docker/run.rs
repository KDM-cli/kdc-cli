use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub image: String,
    pub name: Option<String>,
    pub ports: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub detached: bool,
}

impl Default for RunRequest {
    fn default() -> Self {
        Self {
            image: String::new(),
            name: None,
            ports: Vec::new(),
            env_vars: Vec::new(),
            detached: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    pub container_id: String,
    pub success: bool,
    pub output: String,
}

/// Run a Docker container from the given image.
pub fn execute(request: &RunRequest) -> Result<RunResult> {
    let mut args = vec!["run".to_string()];

    if request.detached {
        args.push("-d".to_string());
    }

    if let Some(name) = &request.name {
        args.push("--name".to_string());
        args.push(name.clone());
    }

    for port in &request.ports {
        args.push("-p".to_string());
        args.push(port.clone());
    }

    for (key, value) in &request.env_vars {
        args.push("-e".to_string());
        args.push(format!("{key}={value}"));
    }

    args.push(request.image.clone());

    let output = Command::new("docker")
        .args(&args)
        .output()
        .context("Failed to execute docker run")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    Ok(RunResult {
        container_id: stdout.clone(),
        success: output.status.success(),
        output: if output.status.success() {
            stdout
        } else {
            stderr
        },
    })
}

/// Stop a running Docker container.
pub fn stop(container_id: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["stop", container_id])
        .output()
        .context("Failed to execute docker stop")?;

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        Ok(result)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker stop failed: {err}")
    }
}

/// Restart a Docker container.
pub fn restart(container_id: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["restart", container_id])
        .output()
        .context("Failed to execute docker restart")?;

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        Ok(result)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker restart failed: {err}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_request_defaults_to_detached() {
        let request = RunRequest {
            image: "nginx:latest".to_string(),
            ..Default::default()
        };
        assert!(request.detached);
    }

    #[test]
    fn run_request_builds_with_ports_and_env() {
        let request = RunRequest {
            image: "myapp:latest".to_string(),
            name: Some("my-container".to_string()),
            ports: vec!["8080:80".to_string()],
            env_vars: vec![("NODE_ENV".to_string(), "production".to_string())],
            detached: true,
        };
        assert_eq!(request.ports.len(), 1);
        assert_eq!(request.env_vars.len(), 1);
        assert_eq!(request.name, Some("my-container".to_string()));
    }
}
