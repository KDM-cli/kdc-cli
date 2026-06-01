use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerImage {
    pub repository: String,
    pub tag: String,
    pub image_id: String,
    pub size: String,
}

impl DockerImage {
    pub fn full_name(&self) -> String {
        if self.tag.is_empty() || self.tag == "<none>" {
            self.repository.clone()
        } else {
            format!("{}:{}", self.repository, self.tag)
        }
    }
}

/// List Docker images on the local machine.
pub fn list() -> Result<Vec<DockerImage>> {
    let output = Command::new("docker")
        .args([
            "images",
            "--format",
            "{{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.Size}}",
        ])
        .output()
        .context("Failed to execute docker images")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker images failed: {err}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let images = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() >= 4 {
                Some(DockerImage {
                    repository: parts[0].to_string(),
                    tag: parts[1].to_string(),
                    image_id: parts[2].to_string(),
                    size: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(images)
}

/// Tag a Docker image with a new tag.
pub fn tag(source: &str, new_tag: &str) -> Result<()> {
    let output = Command::new("docker")
        .args(["tag", source, new_tag])
        .output()
        .context("Failed to execute docker tag")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker tag failed: {err}");
    }

    Ok(())
}

/// Delete a Docker image.
pub fn delete(image: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["rmi", image])
        .output()
        .context("Failed to execute docker rmi")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker rmi failed: {err}")
    }
}

/// Push a Docker image to a registry.
pub fn push(image: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["push", image])
        .output()
        .context("Failed to execute docker push")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker push failed: {err}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_image_full_name() {
        let image = DockerImage {
            repository: "myapp".to_string(),
            tag: "v1.0".to_string(),
            image_id: "sha256:abc".to_string(),
            size: "150MB".to_string(),
        };
        assert_eq!(image.full_name(), "myapp:v1.0");
    }

    #[test]
    fn docker_image_full_name_without_tag() {
        let image = DockerImage {
            repository: "myapp".to_string(),
            tag: "<none>".to_string(),
            image_id: "sha256:abc".to_string(),
            size: "150MB".to_string(),
        };
        assert_eq!(image.full_name(), "myapp");
    }

    #[test]
    fn test_image_ops() {
        crate::utils::test_support::set_mock_path();
        
        let images = list().unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].repository, "myapp");

        assert!(tag("myapp:latest", "myapp:v2").is_ok());
        assert!(delete("myapp:latest").is_ok());
        assert!(push("myapp:latest").is_ok());
    }
}
