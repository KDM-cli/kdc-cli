use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeDownRequest {
    pub remove_volumes: bool,
    pub remove_orphans: bool,
}

impl Default for ComposeDownRequest {
    fn default() -> Self {
        Self {
            remove_volumes: false,
            remove_orphans: true,
        }
    }
}

/// Execute `docker compose down`.
pub fn execute(request: &ComposeDownRequest, project_root: &Path) -> Result<String> {
    let mut args = vec!["compose".to_string(), "down".to_string()];

    if request.remove_volumes {
        args.push("-v".to_string());
    }

    if request.remove_orphans {
        args.push("--remove-orphans".to_string());
    }

    let output = Command::new("docker")
        .args(&args)
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker compose down")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(format!("{stdout}{stderr}"))
    } else {
        anyhow::bail!("docker compose down failed: {stderr}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_down_defaults() {
        let request = ComposeDownRequest::default();
        assert!(!request.remove_volumes);
        assert!(request.remove_orphans);
    }

    #[test]
    fn test_execute() {
        crate::utils::test_support::set_mock_path();
        let request = ComposeDownRequest {
            remove_volumes: true,
            remove_orphans: true,
        };
        let res = execute(&request, Path::new("."));
        assert!(res.is_ok());
        let output = res.unwrap();
        assert!(output.contains("Stopping container"));
    }
}
