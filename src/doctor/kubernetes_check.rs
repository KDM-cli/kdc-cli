use std::process::Command;

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KubernetesStatus {
    Unknown,
    Connected,
    Disconnected,
}

/// Check if a Kubernetes cluster is reachable.
pub fn check_cluster() -> KubernetesStatus {
    match Command::new("kubectl").args(["cluster-info"]).output() {
        Ok(output) if output.status.success() => KubernetesStatus::Connected,
        Ok(_) => KubernetesStatus::Disconnected,
        Err(_) => KubernetesStatus::Unknown,
    }
}

/// Get the currently active kubectl context name.
pub fn current_context() -> Option<String> {
    Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// List the names of nodes in the current cluster.
pub fn check_nodes() -> Result<Vec<String>> {
    let output = Command::new("kubectl")
        .args(["get", "nodes", "-o", "jsonpath={.items[*].metadata.name}"])
        .output()
        .context("Failed to execute kubectl get nodes")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("kubectl get nodes failed: {err}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let nodes = stdout
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kubernetes_status_variants() {
        assert_ne!(KubernetesStatus::Connected, KubernetesStatus::Disconnected);
        assert_ne!(KubernetesStatus::Unknown, KubernetesStatus::Connected);
    }

    #[test]
    fn test_check_cluster_and_nodes() {
        crate::utils::test_support::set_mock_path();

        let status = check_cluster();
        assert_eq!(status, KubernetesStatus::Connected);

        let ctx = current_context().unwrap();
        assert_eq!(ctx, "minikube");

        let nodes = check_nodes().unwrap();
        assert_eq!(nodes, vec!["node1", "node2"]);
    }
}
