use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// List the services defined in the Compose file.
pub fn list(project_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("docker")
        .args(["compose", "config", "--services"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker compose config --services")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("docker compose config --services failed: {err}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let services = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(services)
}

/// Check if any Compose services are currently running.
pub fn running(project_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("docker")
        .args(["compose", "ps", "--services", "--filter", "status=running"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker compose ps --services")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let services = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(services)
}

#[cfg(test)]
mod tests {
    #[test]
    fn services_module_compiles() {
        // This module relies on the `docker compose` CLI, so we only verify
        // the module compiles correctly in unit tests.
        let _: fn(&std::path::Path) -> anyhow::Result<Vec<String>> = super::list;
    }
}
