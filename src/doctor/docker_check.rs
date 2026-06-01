use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerStatus {
    Unknown,
    Running,
    Unavailable,
}

/// Check if the Docker daemon is currently running.
pub fn check_daemon() -> DockerStatus {
    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => DockerStatus::Running,
        Ok(_) => DockerStatus::Unavailable,
        Err(_) => DockerStatus::Unknown,
    }
}

/// Retrieve the installed Docker version string, if available.
pub fn check_version() -> Option<String> {
    Command::new("docker")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_status_variants() {
        // Just verify the enum variants exist and can be compared.
        assert_ne!(DockerStatus::Running, DockerStatus::Unavailable);
        assert_ne!(DockerStatus::Unknown, DockerStatus::Running);
    }
}
