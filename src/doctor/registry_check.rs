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

/// Check if a container registry is reachable by verifying Docker connectivity.
pub fn check_registry(registry: &str) -> RegistryStatus {
    // We use `docker info` as a basic connectivity check. A more thorough check
    // would attempt to pull a manifest, but that requires auth for private registries.
    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => {
            // Docker daemon is reachable; registry check is best-effort.
            if registry.is_empty() {
                RegistryStatus::Unknown
            } else {
                RegistryStatus::Connected
            }
        }
        Ok(_) => RegistryStatus::Disconnected,
        Err(_) => RegistryStatus::Unknown,
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
}
