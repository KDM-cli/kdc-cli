use anyhow::{Context, Result};
use std::process::Command;

pub fn run_kubectl(args: &[&str]) -> Result<String> {
    let output = Command::new("kubectl")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run kubectl {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("kubectl {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn non_empty_lines(output: &str) -> impl Iterator<Item = &str> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
}

pub fn kubectl_list(resource: &str, namespace: Option<&str>) -> Result<String> {
    let mut args = vec!["get", resource, "--no-headers"];
    if let Some(namespace) = namespace {
        args.push("-n");
        args.push(namespace);
    }

    run_kubectl(&args)
}

#[cfg(test)]
mod tests {
    use super::non_empty_lines;

    #[test]
    fn trims_empty_lines() {
        let lines = non_empty_lines(" a \n\n b\n").collect::<Vec<_>>();
        assert_eq!(lines, vec!["a", "b"]);
    }
}
