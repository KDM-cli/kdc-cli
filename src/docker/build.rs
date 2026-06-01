use std::path::Path;
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRequest {
    pub image: String,
    pub tag: String,
}

impl BuildRequest {
    pub fn full_tag(&self) -> String {
        format!("{}:{}", self.image, self.tag)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildResult {
    pub success: bool,
    pub image_tag: String,
    pub output: String,
    pub duration_secs: u64,
}

fn docker_build_with_args(
    request: &BuildRequest,
    project_root: &Path,
    extra_args: &[&str],
) -> Result<BuildResult> {
    let full_tag = request.full_tag();
    let start = Instant::now();

    let mut args = vec!["build"];
    args.extend_from_slice(extra_args);
    args.extend_from_slice(&["-t", &full_tag, "."]);

    let output = Command::new("docker")
        .args(&args)
        .current_dir(project_root)
        .output()
        .context("Failed to execute docker build")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    Ok(BuildResult {
        success: output.status.success(),
        image_tag: full_tag,
        output: combined,
        duration_secs: start.elapsed().as_secs(),
    })
}

/// Build a Docker image from the Dockerfile in `project_root`.
pub fn execute(request: &BuildRequest, project_root: &Path) -> Result<BuildResult> {
    docker_build_with_args(request, project_root, &[])
}

/// Rebuild a Docker image (equivalent to build with `--no-cache`).
pub fn rebuild(request: &BuildRequest, project_root: &Path) -> Result<BuildResult> {
    docker_build_with_args(request, project_root, &["--no-cache"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_formats_full_tag() {
        let request = BuildRequest {
            image: "myapp".to_string(),
            tag: "latest".to_string(),
        };
        assert_eq!(request.full_tag(), "myapp:latest");
    }

    #[test]
    fn build_request_with_registry() {
        let request = BuildRequest {
            image: "registry.io/myapp".to_string(),
            tag: "v1.0.0".to_string(),
        };
        assert_eq!(request.full_tag(), "registry.io/myapp:v1.0.0");
    }

    #[test]
    fn test_execute_and_rebuild() {
        crate::utils::test_support::set_mock_path();
        let request = BuildRequest {
            image: "myapp".to_string(),
            tag: "latest".to_string(),
        };
        let res = execute(&request, Path::new(".")).unwrap();
        assert!(res.success);
        assert_eq!(res.image_tag, "myapp:latest");

        let res_rebuild = rebuild(&request, Path::new(".")).unwrap();
        assert!(res_rebuild.success);
        assert_eq!(res_rebuild.image_tag, "myapp:latest");
    }
}
