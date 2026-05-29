use std::process::Command;

use crate::project::{ProjectCapabilities, RuntimeCapabilities};

pub fn detect(capabilities: &ProjectCapabilities) -> RuntimeCapabilities {
    RuntimeCapabilities {
        docker_running: capabilities.docker && command_succeeds("docker", &["info"]),
        cluster_connected: capabilities.kubernetes && kubernetes_available(),
        registry_connected: false,
        deployment_exists: false,
        rollback_available: false,
    }
}

fn kubernetes_available() -> bool {
    command_succeeds("kubectl", &["cluster-info"])
        || (!command_exists("kubectl") && command_succeeds("minikube", &["status"]))
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_succeeds(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_defaults_are_false_without_project_capabilities() {
        let runtime = detect(&ProjectCapabilities::default());

        assert!(!runtime.docker_running);
        assert!(!runtime.cluster_connected);
        assert!(!runtime.registry_connected);
    }
}
