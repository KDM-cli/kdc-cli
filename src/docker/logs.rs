use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerLogLine {
    pub message: String,
}

/// Fetch the last `tail` lines of logs from a Docker container.
pub fn fetch(container_id: &str, tail: usize) -> Result<Vec<DockerLogLine>> {
    let tail_str = tail.to_string();
    let output = Command::new("docker")
        .args(["logs", "--tail", &tail_str, container_id])
        .output()
        .context("Failed to execute docker logs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker logs failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Docker sends some log output to stderr, so combine both streams.
    let combined = if stdout.ends_with('\n') || stdout.is_empty() {
        format!("{stdout}{stderr}")
    } else {
        format!("{stdout}\n{stderr}")
    };

    let lines = combined
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| DockerLogLine {
            message: line.to_string(),
        })
        .collect();

    Ok(lines)
}

/// Fetch all logs from a Docker container.
pub fn fetch_all(container_id: &str) -> Result<Vec<DockerLogLine>> {
    let output = Command::new("docker")
        .args(["logs", container_id])
        .output()
        .context("Failed to execute docker logs")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker logs failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Docker sends some log output to stderr, so combine both streams.
    let combined = if stdout.ends_with('\n') || stdout.is_empty() {
        format!("{stdout}{stderr}")
    } else {
        format!("{stdout}\n{stderr}")
    };

    let lines = combined
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| DockerLogLine {
            message: line.to_string(),
        })
        .collect();

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_log_line_holds_message() {
        let line = DockerLogLine {
            message: "Server started on port 8080".to_string(),
        };
        assert_eq!(line.message, "Server started on port 8080");
    }

    #[test]
    fn test_fetch_and_fetch_all() {
        crate::utils::test_support::set_mock_path();

        let logs = fetch("container123", 10).unwrap();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "line1");

        let all_logs = fetch_all("container123").unwrap();
        assert_eq!(all_logs.len(), 3);
    }
}
