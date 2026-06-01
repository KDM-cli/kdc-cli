use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRequest {
    pub deployment_name: Option<String>,
    pub target_revision: Option<String>,
}

/// Execute a kubectl rollout undo for the given deployment.
pub fn execute(request: &RollbackRequest, namespace: &str) -> Result<String> {
    let deployment = request.deployment_name.as_deref().unwrap_or("deployment");

    let mut args = vec![
        "rollout".to_string(),
        "undo".to_string(),
        format!("deployment/{deployment}"),
        "-n".to_string(),
        namespace.to_string(),
    ];

    if let Some(revision) = &request.target_revision {
        args.push(format!("--to-revision={revision}"));
    }

    let output = Command::new("kubectl")
        .args(&args)
        .output()
        .context("Failed to execute kubectl rollout undo")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(format!("Rollback completed: {stdout}"))
    } else {
        anyhow::bail!("Rollback failed: {stderr}")
    }
}

/// Check rollout history for a deployment.
pub fn history(deployment_name: &str, namespace: &str) -> Result<String> {
    let output = Command::new("kubectl")
        .args([
            "rollout",
            "history",
            &format!("deployment/{deployment_name}"),
            "-n",
            namespace,
        ])
        .output()
        .context("Failed to execute kubectl rollout history")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("rollout history failed: {stderr}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_request_with_revision() {
        let request = RollbackRequest {
            deployment_name: Some("my-app".to_string()),
            target_revision: Some("3".to_string()),
        };
        assert_eq!(request.deployment_name, Some("my-app".to_string()));
        assert_eq!(request.target_revision, Some("3".to_string()));
    }

    #[test]
    fn rollback_request_without_revision() {
        let request = RollbackRequest {
            deployment_name: None,
            target_revision: None,
        };
        assert!(request.deployment_name.is_none());
        assert!(request.target_revision.is_none());
    }

    #[test]
    fn test_execute_and_history() {
        crate::utils::test_support::set_mock_path();

        let request = RollbackRequest {
            deployment_name: Some("my-app".to_string()),
            target_revision: Some("2".to_string()),
        };
        let res = execute(&request, "default").unwrap();
        assert!(res.contains("rolled back"));

        let hist = history("my-app", "default").unwrap();
        assert!(hist.contains("REVISION"));
    }
}
