use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};

use crate::project::{ProjectCapabilities, ProjectContext, RuntimeCapabilities};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStep {
    Build,
    DockerBuild,
    DockerPush,
    DeploymentUpdate,
    RolloutVerification,
}

pub trait DeploymentPipeline {
    fn execute(&self) -> Result<Vec<PipelineStep>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentPlan {
    pub steps: Vec<PipelineStep>,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineStepResult {
    pub step: PipelineStep,
    pub success: bool,
    pub message: String,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExecution {
    pub results: Vec<PipelineStepResult>,
    pub overall_success: bool,
}

impl PipelineExecution {
    pub fn total_duration_secs(&self) -> f64 {
        self.results.iter().map(|r| r.duration_secs).sum()
    }

    pub fn render(&self) -> String {
        let mut lines = vec![format!(
            "Pipeline Execution: {}",
            if self.overall_success {
                "SUCCESS"
            } else {
                "FAILED"
            }
        )];

        for result in &self.results {
            let marker = if result.success { "✓" } else { "✗" };
            lines.push(format!(
                "  {marker} {} ({:.1}s) - {}",
                result.step.label(),
                result.duration_secs,
                result.message
            ));
        }

        lines.push(format!("Total: {:.1}s", self.total_duration_secs()));
        lines.join("\n")
    }
}

impl DeploymentPlan {
    pub fn ready(&self) -> bool {
        self.blockers.is_empty()
    }

    pub fn render(&self) -> String {
        let steps = if self.steps.is_empty() {
            "  - none".to_string()
        } else {
            self.steps
                .iter()
                .map(|step| format!("  - {}", step.label()))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let blockers = if self.blockers.is_empty() {
            "  - none".to_string()
        } else {
            self.blockers
                .iter()
                .map(|blocker| format!("  - {blocker}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            "Deployment Plan\nReady: {}\nSteps:\n{}\nBlockers:\n{}",
            self.ready(),
            steps,
            blockers
        )
    }
}

impl PipelineStep {
    pub fn label(self) -> &'static str {
        match self {
            PipelineStep::Build => "Build Application",
            PipelineStep::DockerBuild => "Docker Build",
            PipelineStep::DockerPush => "Docker Push",
            PipelineStep::DeploymentUpdate => "Deployment Update",
            PipelineStep::RolloutVerification => "Rollout Verification",
        }
    }
}

pub fn plan(capabilities: &ProjectCapabilities, runtime: &RuntimeCapabilities) -> DeploymentPlan {
    let mut steps = Vec::new();
    let mut blockers = Vec::new();

    steps.push(PipelineStep::Build);

    if capabilities.docker {
        steps.push(PipelineStep::DockerBuild);
        if runtime.docker_running {
            steps.push(PipelineStep::DockerPush);
        } else {
            blockers.push("Docker daemon is not running".to_string());
        }
    } else {
        blockers.push("Dockerfile is missing".to_string());
    }

    if capabilities.kubernetes {
        if runtime.cluster_connected {
            steps.push(PipelineStep::DeploymentUpdate);
            steps.push(PipelineStep::RolloutVerification);
        } else {
            blockers.push("Kubernetes cluster is not connected".to_string());
        }
    } else {
        blockers.push("Kubernetes manifests are missing".to_string());
    }

    DeploymentPlan { steps, blockers }
}

/// Execute the deployment pipeline against a real project.
pub fn execute_pipeline(
    plan: &DeploymentPlan,
    project: &ProjectContext,
    capabilities: &ProjectCapabilities,
) -> Result<PipelineExecution> {
    if !plan.ready() {
        anyhow::bail!("Deployment plan has blockers: {}", plan.blockers.join(", "));
    }

    let mut results = Vec::new();
    let mut overall_success = true;

    for step in &plan.steps {
        let start = Instant::now();
        let step_result = match step {
            PipelineStep::Build => execute_build_step(project),
            PipelineStep::DockerBuild => execute_docker_build_step(project),
            PipelineStep::DockerPush => execute_docker_push_step(project),
            PipelineStep::DeploymentUpdate => execute_deployment_update_step(project, capabilities),
            PipelineStep::RolloutVerification => execute_rollout_verification_step(),
        };
        let duration_secs = start.elapsed().as_secs_f64();

        match step_result {
            Ok(message) => {
                results.push(PipelineStepResult {
                    step: *step,
                    success: true,
                    message,
                    duration_secs,
                });
            }
            Err(err) => {
                overall_success = false;
                results.push(PipelineStepResult {
                    step: *step,
                    success: false,
                    message: err.to_string(),
                    duration_secs,
                });
                // Stop the pipeline on first failure.
                break;
            }
        }
    }

    Ok(PipelineExecution {
        results,
        overall_success,
    })
}

fn execute_build_step(project: &ProjectContext) -> Result<String> {
    let build_cmd =
        crate::templates::stacks::build_command(project.stack).unwrap_or("echo 'No build step'");

    let parts: Vec<&str> = build_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok("No build command for this stack".to_string());
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(&project.root)
        .output()
        .with_context(|| format!("Failed to execute build command: {build_cmd}"))?;

    if output.status.success() {
        Ok(format!("Build completed: {build_cmd}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Build failed: {stderr}")
    }
}

fn execute_docker_build_step(project: &ProjectContext) -> Result<String> {
    let image_name = project.name.to_lowercase().replace(' ', "-");
    let request = crate::docker::build::BuildRequest {
        image: image_name.clone(),
        tag: "latest".to_string(),
    };

    let result = crate::docker::build::execute(&request, &project.root)?;
    if result.success {
        Ok(format!("Docker image built: {}", result.image_tag))
    } else {
        anyhow::bail!("Docker build failed: {}", result.output)
    }
}

fn execute_docker_push_step(project: &ProjectContext) -> Result<String> {
    let image_name = project.name.to_lowercase().replace(' ', "-");
    let full_tag = format!("{image_name}:latest");

    crate::docker::images::push(&full_tag)?;
    Ok(format!("Pushed {full_tag}"))
}

fn execute_deployment_update_step(
    project: &ProjectContext,
    capabilities: &ProjectCapabilities,
) -> Result<String> {
    if !capabilities.kubernetes {
        return Ok("No Kubernetes manifests to apply".to_string());
    }

    // Apply all detected Kubernetes manifests.
    let k8s_dir = project.root.join("k8s");
    let manifest_path = if k8s_dir.exists() {
        k8s_dir.to_string_lossy().to_string()
    } else {
        project.root.to_string_lossy().to_string()
    };

    let output = Command::new("kubectl")
        .args(["apply", "-f", &manifest_path])
        .output()
        .context("Failed to execute kubectl apply")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("Deployment updated: {stdout}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("kubectl apply failed: {stderr}")
    }
}

fn execute_rollout_verification_step() -> Result<String> {
    let output = Command::new("kubectl")
        .args(["rollout", "status", "deployment", "--timeout=120s"])
        .output()
        .context("Failed to execute kubectl rollout status")?;

    if output.status.success() {
        Ok("Rollout verified successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Rollout verification failed: {stderr}")
    }
}

#[cfg(test)]
mod tests {
    use crate::project::{ProjectCapabilities, RuntimeCapabilities};

    use super::{plan, PipelineStep};

    #[test]
    fn full_plan_is_ready_when_capabilities_and_runtime_are_available() {
        let plan = plan(
            &ProjectCapabilities {
                docker: true,
                kubernetes: true,
                deployment: true,
                ..ProjectCapabilities::default()
            },
            &RuntimeCapabilities {
                docker_running: true,
                cluster_connected: true,
                ..RuntimeCapabilities::default()
            },
        );

        assert!(plan.ready());
        assert_eq!(
            plan.steps,
            vec![
                PipelineStep::Build,
                PipelineStep::DockerBuild,
                PipelineStep::DockerPush,
                PipelineStep::DeploymentUpdate,
                PipelineStep::RolloutVerification,
            ]
        );
    }

    #[test]
    fn plan_has_blockers_without_docker() {
        let plan = plan(
            &ProjectCapabilities::default(),
            &RuntimeCapabilities::default(),
        );

        assert!(!plan.ready());
        assert!(plan.blockers.contains(&"Dockerfile is missing".to_string()));
    }

    #[test]
    fn pipeline_execution_renders() {
        use super::{PipelineExecution, PipelineStepResult};

        let execution = PipelineExecution {
            results: vec![
                PipelineStepResult {
                    step: PipelineStep::Build,
                    success: true,
                    message: "done".to_string(),
                    duration_secs: 2.5,
                },
                PipelineStepResult {
                    step: PipelineStep::DockerBuild,
                    success: false,
                    message: "Dockerfile not found".to_string(),
                    duration_secs: 0.1,
                },
            ],
            overall_success: false,
        };

        let rendered = execution.render();
        assert!(rendered.contains("FAILED"));
        assert!(rendered.contains("✓ Build Application"));
        assert!(rendered.contains("✗ Docker Build"));
    }
}
