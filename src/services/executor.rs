use std::path::Path;

use anyhow::Result;

use crate::{compose, config, docker, project::ProjectContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
    pub output_lines: Vec<String>,
}

pub trait CommandExecutor {
    fn execute(&self, action_id: &str) -> Result<ExecutionResult>;
}

/// The main executor that dispatches action IDs to module functions.
pub struct KdcExecutor<'a> {
    pub project: &'a ProjectContext,
}

impl<'a> KdcExecutor<'a> {
    pub fn new(project: &'a ProjectContext) -> Self {
        Self { project }
    }
}

impl<'a> CommandExecutor for KdcExecutor<'a> {
    fn execute(&self, action_id: &str) -> Result<ExecutionResult> {
        match action_id {
            "docker.build" => self.docker_build(),
            "docker.run" => self.docker_run(),
            "docker.logs" => self.docker_logs(),
            "compose.up" => self.compose_up(),
            "compose.down" => self.compose_down(),
            "compose.logs" => self.compose_logs(),
            "project.analysis" => self.project_analysis(),
            "settings.open" => Ok(ExecutionResult {
                success: true,
                message: "Settings screen opened".to_string(),
                output_lines: Vec::new(),
            }),
            _ => Ok(ExecutionResult {
                success: false,
                message: format!("Unknown action: {action_id}"),
                output_lines: Vec::new(),
            }),
        }
    }
}

impl<'a> KdcExecutor<'a> {
    fn docker_build(&self) -> Result<ExecutionResult> {
        let request = docker::build::BuildRequest {
            image: self.project.name.to_lowercase().replace(' ', "-"),
            tag: "latest".to_string(),
        };

        let result = docker::build::execute(&request, &self.project.root)?;
        let lines: Vec<String> = result.output.lines().map(|l| l.to_string()).collect();

        Ok(ExecutionResult {
            success: result.success,
            message: if result.success {
                format!("Built {}", result.image_tag)
            } else {
                "Docker build failed".to_string()
            },
            output_lines: lines,
        })
    }

    fn docker_run(&self) -> Result<ExecutionResult> {
        let image = format!(
            "{}:latest",
            self.project.name.to_lowercase().replace(' ', "-")
        );
        let request = docker::run::RunRequest {
            image,
            ..Default::default()
        };

        let result = docker::run::execute(&request)?;
        Ok(ExecutionResult {
            success: result.success,
            message: if result.success {
                format!("Container started: {}", result.container_id)
            } else {
                format!("Failed to start container: {}", result.output)
            },
            output_lines: vec![result.output],
        })
    }

    fn docker_logs(&self) -> Result<ExecutionResult> {
        let containers = docker::containers::list()?;
        if containers.is_empty() {
            return Ok(ExecutionResult {
                success: true,
                message: "No running containers".to_string(),
                output_lines: Vec::new(),
            });
        }

        let first = &containers[0];
        let logs = docker::logs::fetch(&first.id, 50)?;
        let lines: Vec<String> = logs.iter().map(|l| l.message.clone()).collect();

        Ok(ExecutionResult {
            success: true,
            message: format!("Logs from container: {}", first.name),
            output_lines: lines,
        })
    }

    fn compose_up(&self) -> Result<ExecutionResult> {
        let request = compose::up::ComposeUpRequest::default();
        let result = compose::up::execute(&request, &self.project.root)?;
        let lines: Vec<String> = result.output.lines().map(|l| l.to_string()).collect();

        Ok(ExecutionResult {
            success: result.success,
            message: if result.success {
                "Compose up completed".to_string()
            } else {
                "Compose up failed".to_string()
            },
            output_lines: lines,
        })
    }

    fn compose_down(&self) -> Result<ExecutionResult> {
        let request = compose::down::ComposeDownRequest::default();
        let output = compose::down::execute(&request, &self.project.root)?;
        let lines: Vec<String> = output.lines().map(|l| l.to_string()).collect();

        Ok(ExecutionResult {
            success: true,
            message: "Compose down completed".to_string(),
            output_lines: lines,
        })
    }

    fn compose_logs(&self) -> Result<ExecutionResult> {
        let request = compose::logs::ComposeLogRequest::default();
        let lines = compose::logs::fetch(&request, &self.project.root)?;

        Ok(ExecutionResult {
            success: true,
            message: "Compose logs fetched".to_string(),
            output_lines: lines,
        })
    }

    fn project_analysis(&self) -> Result<ExecutionResult> {
        let capabilities = config::settings::Settings::default();
        Ok(ExecutionResult {
            success: true,
            message: format!("Project: {} ({})", self.project.name, self.project.stack),
            output_lines: vec![
                format!("Root: {}", self.project.root.display()),
                format!("Stack: {}", self.project.stack),
                format!("Assets: {}", self.project.assets.len()),
                format!("Theme: {}", capabilities.theme),
            ],
        })
    }
}

/// Convenience function: run an action by ID and return the path to the
/// project root. Useful for modules that need the project directory.
pub fn project_root(project: &ProjectContext) -> &Path {
    &project.root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_result_fields() {
        let result = ExecutionResult {
            success: true,
            message: "done".to_string(),
            output_lines: vec!["line1".to_string(), "line2".to_string()],
        };
        assert!(result.success);
        assert_eq!(result.output_lines.len(), 2);
    }

    #[test]
    fn unknown_action_returns_failure() {
        use crate::domain::project::ProjectStack;
        use std::path::PathBuf;

        let project = ProjectContext {
            name: "test".to_string(),
            root: PathBuf::from("."),
            stack: ProjectStack::Unknown,
            assets: Vec::new(),
        };
        let executor = KdcExecutor::new(&project);
        let result = executor.execute("unknown.action").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Unknown action"));
    }
}
