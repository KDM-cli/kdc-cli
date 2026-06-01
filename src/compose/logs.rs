use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeLogRequest {
    pub follow: bool,
    pub service: Option<String>,
    pub tail: Option<usize>,
}

impl Default for ComposeLogRequest {
    fn default() -> Self {
        Self {
            follow: false,
            service: None,
            tail: Some(100),
        }
    }
}

/// Fetch compose logs.
pub fn fetch(request: &ComposeLogRequest, project_root: &Path) -> Result<Vec<String>> {
    let mut args = vec!["compose".to_string(), "logs".to_string()];

    if request.follow {
        args.push("--follow".to_string());
    }

    if let Some(tail) = request.tail {
        args.push("--tail".to_string());
        args.push(tail.to_string());
    }

    if let Some(service) = &request.service {
        args.push(service.clone());
    }

    let output = Command::new("docker")
        .args(&args)
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker compose logs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker compose logs failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    let lines = combined
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_log_request_defaults() {
        let request = ComposeLogRequest::default();
        assert!(!request.follow);
        assert!(request.service.is_none());
        assert_eq!(request.tail, Some(100));
    }

    #[test]
    fn test_fetch_compose_logs() {
        crate::utils::test_support::set_mock_path();
        let request = ComposeLogRequest {
            follow: false,
            service: Some("web".to_string()),
            tail: Some(10),
        };
        let res = fetch(&request, Path::new("."));
        assert!(res.is_ok());
        let lines = res.unwrap();
        assert!(lines.len() >= 2);
        assert_eq!(lines[0], "compose log line 1");
    }
}
