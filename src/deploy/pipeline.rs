use anyhow::Result;

use crate::project::{ProjectCapabilities, RuntimeCapabilities};

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
}
