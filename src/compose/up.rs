use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeUpRequest {
    pub detached: bool,
    pub services: Vec<String>,
    pub build: bool,
}

impl Default for ComposeUpRequest {
    fn default() -> Self {
        Self {
            detached: true,
            services: Vec::new(),
            build: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeUpResult {
    pub success: bool,
    pub output: String,
}

/// Execute `docker compose up`.
pub fn execute(request: &ComposeUpRequest, project_root: &Path) -> Result<ComposeUpResult> {
    let mut args = vec!["compose".to_string(), "up".to_string()];

    if request.detached {
        args.push("-d".to_string());
    }

    if request.build {
        args.push("--build".to_string());
    }

    for service in &request.services {
        args.push(service.clone());
    }

    let output = Command::new("docker")
        .args(&args)
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker compose up")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    Ok(ComposeUpResult {
        success: output.status.success(),
        output: format!("{stdout}{stderr}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_up_defaults_to_detached() {
        let request = ComposeUpRequest::default();
        assert!(request.detached);
        assert!(request.services.is_empty());
        assert!(!request.build);
    }

    #[test]
    fn test_execute() {
        crate::utils::test_support::set_mock_path();
        let request = ComposeUpRequest {
            detached: true,
            services: vec!["db".to_string()],
            build: true,
        };
        let res = execute(&request, Path::new("."));
        assert!(res.is_ok());
        let result = res.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Starting container"));
    }
}
