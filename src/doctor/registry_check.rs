use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryStatus {
    Unknown,
    Connected,
    Disconnected,
}

/// Configuration for a container registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,
    pub username: Option<String>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: "docker.io".to_string(),
            username: None,
        }
    }
}

fn run_command_with_timeout(
    cmd: &str,
    args: &[&str],
    timeout: std::time::Duration,
) -> anyhow::Result<std::process::Output> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let start = std::time::Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            let output = child.wait_with_output()?;
            return Ok(output);
        }
        if start.elapsed() >= timeout {
            child.kill()?;
            anyhow::bail!("Command timed out after {:?}", timeout);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Check if a container registry is reachable by verifying Docker connectivity.
fn build_v2_url(registry: &str) -> String {
    let domain = registry
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    let target_domain = if domain == "docker.io" {
        "registry-1.docker.io"
    } else {
        domain
    };

    format!("https://{}/v2/", target_domain)
}

fn probe_registry_endpoint(url: &str) -> Option<RegistryStatus> {
    match ureq::head(url)
        .timeout(std::time::Duration::from_secs(3))
        .call()
    {
        Ok(resp) => {
            let code = resp.status();
            if code == 200 || code == 401 {
                Some(RegistryStatus::Connected)
            } else {
                Some(RegistryStatus::Disconnected)
            }
        }
        Err(ureq::Error::Status(code, _)) => {
            if code == 401 {
                Some(RegistryStatus::Connected)
            } else {
                Some(RegistryStatus::Disconnected)
            }
        }
        Err(_) => None,
    }
}

fn check_docker_fallback() -> bool {
    match run_command_with_timeout("docker", &["info"], std::time::Duration::from_secs(2)) {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Check if a container registry is reachable by verifying Docker connectivity.
pub fn check_registry(registry: &str) -> RegistryStatus {
    if registry.is_empty() {
        return RegistryStatus::Unknown;
    }

    let url = build_v2_url(registry);

    if let Some(status) = probe_registry_endpoint(&url) {
        status
    } else if check_docker_fallback() {
        RegistryStatus::Connected
    } else {
        RegistryStatus::Disconnected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_config_defaults() {
        let config = RegistryConfig::default();
        assert_eq!(config.url, "docker.io");
        assert!(config.username.is_none());
    }

    #[test]
    fn registry_config_with_custom_url() {
        let config = RegistryConfig {
            url: "ghcr.io".to_string(),
            username: Some("user".to_string()),
        };
        assert_eq!(config.url, "ghcr.io");
        assert_eq!(config.username, Some("user".to_string()));
    }

    #[test]
    fn registry_status_variants() {
        assert_ne!(RegistryStatus::Connected, RegistryStatus::Disconnected);
        assert_ne!(RegistryStatus::Unknown, RegistryStatus::Connected);
    }

    #[test]
    fn test_check_registry() {
        crate::utils::test_support::set_mock_path();

        let status_empty = check_registry("");
        assert_eq!(status_empty, RegistryStatus::Unknown);

        // Fallback to docker info since invalid-url will fail HTTP HEAD
        let status_fallback = check_registry("invalid-url");
        assert_eq!(status_fallback, RegistryStatus::Connected);
    }
}
