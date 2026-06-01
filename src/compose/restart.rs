use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeRestartRequest {
    pub service: Option<String>,
}

/// Execute `docker compose restart`.
pub fn execute(request: &ComposeRestartRequest, project_root: &Path) -> Result<String> {
    let mut args = vec!["compose".to_string(), "restart".to_string()];

    if let Some(service) = &request.service {
        args.push(service.clone());
    }

    let output = Command::new("docker")
        .args(&args)
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker compose restart")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(format!("{stdout}{stderr}"))
    } else {
        anyhow::bail!("docker compose restart failed: {stderr}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_restart_without_service() {
        let request = ComposeRestartRequest { service: None };
        assert!(request.service.is_none());
    }

    #[test]
    fn compose_restart_with_service() {
        let request = ComposeRestartRequest {
            service: Some("web".to_string()),
        };
        assert_eq!(request.service, Some("web".to_string()));
    }
}
