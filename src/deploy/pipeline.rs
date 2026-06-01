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

pub trait CommandRunner {
    fn run(
        &self,
        cmd: &str,
        args: &[&str],
        current_dir: Option<&std::path::Path>,
    ) -> Result<std::process::Output>;
}

pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(
        &self,
        cmd: &str,
        args: &[&str],
        current_dir: Option<&std::path::Path>,
    ) -> Result<std::process::Output> {
        let mut command = Command::new(cmd);
        command.args(args);
        if let Some(dir) = current_dir {
            command.current_dir(dir);
        }
        command
            .output()
            .context(format!("Failed to execute {}", cmd))
    }
}

/// Execute the deployment pipeline against a real project.
pub fn execute_pipeline(
    plan: &DeploymentPlan,
    project: &ProjectContext,
    capabilities: &ProjectCapabilities,
    environment_str: &str,
) -> Result<PipelineExecution> {
    execute_pipeline_with_runner(
        plan,
        project,
        capabilities,
        environment_str,
        &RealCommandRunner,
    )
}

pub fn execute_pipeline_with_runner(
    plan: &DeploymentPlan,
    project: &ProjectContext,
    capabilities: &ProjectCapabilities,
    environment_str: &str,
    runner: &dyn CommandRunner,
) -> Result<PipelineExecution> {
    if !plan.ready() {
        anyhow::bail!("Deployment plan has blockers: {}", plan.blockers.join(", "));
    }

    let env = crate::deploy::environments::from_string(environment_str);
    let namespace = crate::deploy::environments::resolve_namespace(&env);

    let mut results = Vec::new();
    let mut overall_success = true;

    for step in &plan.steps {
        let start = Instant::now();
        let step_result = match step {
            PipelineStep::Build => execute_build_step(project, runner),
            PipelineStep::DockerBuild => execute_docker_build_step(project),
            PipelineStep::DockerPush => execute_docker_push_step(project),
            PipelineStep::DeploymentUpdate => {
                execute_deployment_update_step(project, capabilities, &namespace, runner)
            }
            PipelineStep::RolloutVerification => {
                let deployment_name = project.name.to_lowercase().replace(' ', "-");
                execute_rollout_verification_step(&deployment_name, &namespace, runner)
            }
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

fn execute_build_step(project: &ProjectContext, runner: &dyn CommandRunner) -> Result<String> {
    let build_cmd =
        crate::templates::stacks::build_command(project.stack).unwrap_or("echo 'No build step'");

    let parts: Vec<&str> = build_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok("No build command for this stack".to_string());
    }

    let output = runner.run(parts[0], &parts[1..], Some(&project.root))?;

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
    namespace: &str,
    runner: &dyn CommandRunner,
) -> Result<String> {
    if !capabilities.kubernetes {
        return Ok("No Kubernetes manifests to apply".to_string());
    }

    let k8s_dir = project.root.join("k8s");
    if !k8s_dir.exists() || !k8s_dir.is_dir() {
        anyhow::bail!("k8s/ directory is absent");
    }

    let manifest_path = k8s_dir.to_string_lossy().to_string();

    let output = runner.run(
        "kubectl",
        &["apply", "-f", &manifest_path, "-n", namespace],
        None,
    )?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(format!("Deployment updated: {stdout}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("kubectl apply failed: {stderr}")
    }
}

fn execute_rollout_verification_step(
    name: &str,
    namespace: &str,
    runner: &dyn CommandRunner,
) -> Result<String> {
    let output = runner.run(
        "kubectl",
        &[
            "rollout",
            "status",
            &format!("deployment/{}", name),
            "-n",
            namespace,
            "--timeout=120s",
        ],
        None,
    )?;

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

    #[test]
    fn plan_blocked_without_cluster() {
        let plan = plan(
            &ProjectCapabilities {
                docker: true,
                kubernetes: true,
                ..ProjectCapabilities::default()
            },
            &RuntimeCapabilities {
                docker_running: true,
                cluster_connected: false,
                ..RuntimeCapabilities::default()
            },
        );

        assert!(!plan.ready());
        assert!(plan
            .blockers
            .contains(&"Kubernetes cluster is not connected".to_string()));
    }

    #[test]
    fn pipeline_execution_render_shows_success() {
        use super::{PipelineExecution, PipelineStepResult};

        let execution = PipelineExecution {
            results: vec![
                PipelineStepResult {
                    step: PipelineStep::Build,
                    success: true,
                    message: "done".to_string(),
                    duration_secs: 1.5,
                },
                PipelineStepResult {
                    step: PipelineStep::DockerBuild,
                    success: true,
                    message: "built".to_string(),
                    duration_secs: 5.0,
                },
            ],
            overall_success: true,
        };

        let rendered = execution.render();
        assert!(rendered.contains("SUCCESS"));
        assert!(rendered.contains("✓ Build Application"));
        assert!(rendered.contains("✓ Docker Build"));
    }

    #[test]
    fn pipeline_execution_total_duration() {
        use super::{PipelineExecution, PipelineStepResult};

        let execution = PipelineExecution {
            results: vec![
                PipelineStepResult {
                    step: PipelineStep::Build,
                    success: true,
                    message: "ok".to_string(),
                    duration_secs: 2.0,
                },
                PipelineStepResult {
                    step: PipelineStep::DockerBuild,
                    success: true,
                    message: "ok".to_string(),
                    duration_secs: 3.5,
                },
            ],
            overall_success: true,
        };

        assert!((execution.total_duration_secs() - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn deployment_plan_render_includes_steps_and_blockers() {
        use super::DeploymentPlan;

        let plan = DeploymentPlan {
            steps: vec![PipelineStep::Build, PipelineStep::DockerBuild],
            blockers: vec!["Docker daemon is not running".to_string()],
        };

        let rendered = plan.render();
        assert!(rendered.contains("Build Application"));
        assert!(rendered.contains("Docker Build"));
        assert!(rendered.contains("Docker daemon is not running"));
        assert!(rendered.contains("Ready: false"));
    }
}

#[cfg(test)]
mod pipeline_mock_tests {
    use super::*;
    use std::sync::Mutex;

    struct MockRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        success: bool,
    }

    impl CommandRunner for MockRunner {
        fn run(
            &self,
            cmd: &str,
            args: &[&str],
            _current_dir: Option<&std::path::Path>,
        ) -> Result<std::process::Output> {
            self.calls.lock().unwrap().push((
                cmd.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            let status = if self.success {
                Command::new("true").status().unwrap()
            } else {
                Command::new("false").status().unwrap()
            };
            Ok(std::process::Output {
                status,
                stdout: b"mock-output".to_vec(),
                stderr: b"mock-error".to_vec(),
            })
        }
    }

    #[test]
    fn test_execute_deployment_update_step() {
        let temp = std::env::temp_dir().join(format!(
            "kdc-k8s-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(temp.join("k8s")).unwrap();

        let project = ProjectContext {
            name: "test-proj".to_string(),
            root: temp.clone(),
            stack: crate::domain::project::ProjectStack::Rust,
            assets: vec![],
        };
        let caps = ProjectCapabilities {
            kubernetes: true,
            ..Default::default()
        };
        let runner = MockRunner {
            calls: Mutex::new(vec![]),
            success: true,
        };

        let result = execute_deployment_update_step(&project, &caps, "my-namespace", &runner);
        assert!(result.is_ok());

        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "kubectl");
        assert!(calls[0].1.contains(&"-n".to_string()));
        assert!(calls[0].1.contains(&"my-namespace".to_string()));
        assert!(calls[0]
            .1
            .contains(&temp.join("k8s").to_string_lossy().to_string()));

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_execute_deployment_update_step_missing_k8s() {
        let temp = std::env::temp_dir().join(format!(
            "kdc-k8s-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp).unwrap(); // no k8s folder

        let project = ProjectContext {
            name: "test-proj".to_string(),
            root: temp.clone(),
            stack: crate::domain::project::ProjectStack::Rust,
            assets: vec![],
        };
        let caps = ProjectCapabilities {
            kubernetes: true,
            ..Default::default()
        };
        let runner = MockRunner {
            calls: Mutex::new(vec![]),
            success: true,
        };

        let result = execute_deployment_update_step(&project, &caps, "my-namespace", &runner);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "k8s/ directory is absent");

        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn test_execute_rollout_verification_step() {
        let runner = MockRunner {
            calls: Mutex::new(vec![]),
            success: true,
        };

        let result = execute_rollout_verification_step("my-app", "my-namespace", &runner);
        assert!(result.is_ok());

        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "kubectl");
        assert!(calls[0].1.contains(&"deployment/my-app".to_string()));
        assert!(calls[0].1.contains(&"-n".to_string()));
        assert!(calls[0].1.contains(&"my-namespace".to_string()));
    }

    #[test]
    fn test_execute_pipeline_with_runner() {
        crate::utils::test_support::set_mock_path();

        let temp = std::env::temp_dir().join(format!(
            "kdc-pipeline-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(temp.join("k8s")).unwrap();

        let project = ProjectContext {
            name: "test-proj".to_string(),
            root: temp.clone(),
            stack: crate::domain::project::ProjectStack::Rust,
            assets: vec![],
        };
        let caps = ProjectCapabilities {
            docker: true,
            kubernetes: true,
            deployment: true,
            ..Default::default()
        };
        let runtime = RuntimeCapabilities {
            docker_running: true,
            cluster_connected: true,
            ..Default::default()
        };

        let plan = plan(&caps, &runtime);
        assert!(plan.ready());

        let runner = MockRunner {
            calls: Mutex::new(vec![]),
            success: true,
        };

        let res =
            execute_pipeline_with_runner(&plan, &project, &caps, "development", &runner).unwrap();
        assert!(res.overall_success);
        assert_eq!(res.results.len(), 5);

        std::fs::remove_dir_all(temp).unwrap();
    }
}
