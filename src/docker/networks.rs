use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
}

/// List Docker networks.
pub fn list() -> Result<Vec<DockerNetwork>> {
    let output = Command::new("docker")
        .args([
            "network",
            "ls",
            "--format",
            "{{.ID}}\t{{.Name}}\t{{.Driver}}\t{{.Scope}}",
        ])
        .output()
        .context("Failed to execute docker network ls")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker network ls failed: {err}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let networks = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() >= 4 {
                Some(DockerNetwork {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    driver: parts[2].to_string(),
                    scope: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(networks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_network_fields() {
        let network = DockerNetwork {
            id: "abc123".to_string(),
            name: "bridge".to_string(),
            driver: "bridge".to_string(),
            scope: "local".to_string(),
        };
        assert_eq!(network.name, "bridge");
        assert_eq!(network.driver, "bridge");
    }

    #[test]
    fn test_list() {
        crate::utils::test_support::set_mock_path();
        let networks = list().unwrap();
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].name, "app-network");
        assert_eq!(networks[0].driver, "bridge");
    }
}
