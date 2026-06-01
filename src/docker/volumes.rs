use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

/// List Docker volumes.
pub fn list() -> Result<Vec<DockerVolume>> {
    let output = Command::new("docker")
        .args([
            "volume",
            "ls",
            "--format",
            "{{.Name}}\t{{.Driver}}\t{{.Mountpoint}}",
        ])
        .output()
        .context("Failed to execute docker volume ls")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker volume ls failed: {err}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let volumes = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() >= 3 {
                Some(DockerVolume {
                    name: parts[0].to_string(),
                    driver: parts[1].to_string(),
                    mountpoint: parts[2].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(volumes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_volume_fields() {
        let volume = DockerVolume {
            name: "my-data".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/my-data/_data".to_string(),
        };
        assert_eq!(volume.name, "my-data");
        assert_eq!(volume.driver, "local");
    }

    #[test]
    fn test_list() {
        crate::utils::test_support::set_mock_path();
        let volumes = list().unwrap();
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].name, "db-data");
        assert_eq!(volumes[0].driver, "local");
    }
}
