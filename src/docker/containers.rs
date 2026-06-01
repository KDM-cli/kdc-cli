use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: String,
}

fn list_with_args(args: &[&str]) -> Result<Vec<ContainerSummary>> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .context("Failed to execute docker ps")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker ps failed: {err}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let containers = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(5, '\t').collect();
            if parts.len() >= 4 {
                Some(ContainerSummary {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    image: parts[2].to_string(),
                    status: parts[3].to_string(),
                    ports: parts.get(4).unwrap_or(&"").to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(containers)
}

/// List running Docker containers.
pub fn list() -> Result<Vec<ContainerSummary>> {
    list_with_args(&[
        "ps",
        "--format",
        "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}",
    ])
}

/// List all Docker containers (including stopped).
pub fn list_all() -> Result<Vec<ContainerSummary>> {
    list_with_args(&[
        "ps",
        "-a",
        "--format",
        "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}",
    ])
}

/// Inspect a container and return the raw JSON output.
pub fn inspect(container_id: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["inspect", container_id])
        .output()
        .context("Failed to execute docker inspect")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker inspect failed: {err}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_summary_fields() {
        let container = ContainerSummary {
            id: "abc123".to_string(),
            name: "my-app".to_string(),
            image: "nginx:latest".to_string(),
            status: "Up 5 minutes".to_string(),
            ports: "0.0.0.0:80->80/tcp".to_string(),
        };
        assert_eq!(container.id, "abc123");
        assert_eq!(container.name, "my-app");
        assert_eq!(container.image, "nginx:latest");
        assert_eq!(container.status, "Up 5 minutes");
        assert_eq!(container.ports, "0.0.0.0:80->80/tcp");
    }

    #[test]
    fn test_list_and_inspect() {
        crate::utils::test_support::set_mock_path();

        let containers = list().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "container123");
        assert_eq!(containers[0].name, "web-app");

        let all_containers = list_all().unwrap();
        assert_eq!(all_containers.len(), 1);

        let details = inspect("container123").unwrap();
        assert_eq!(details, "manifest-info-json"); // from our mock router
    }
}
