use kdc::deploy::{
    history::{DeploymentHistory, DeploymentRecord},
    pipeline::{plan, DeploymentPlan, PipelineExecution, PipelineStep, PipelineStepResult},
};
use kdc::project::{ProjectCapabilities, RuntimeCapabilities};

#[test]
fn plan_ready_with_all_capabilities() {
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
    assert_eq!(plan.steps.len(), 5);
}

#[test]
fn plan_blocked_without_docker() {
    let plan = plan(
        &ProjectCapabilities::default(),
        &RuntimeCapabilities::default(),
    );

    assert!(!plan.ready());
    assert!(plan.blockers.contains(&"Dockerfile is missing".to_string()));
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
fn pipeline_execution_render_shows_failure() {
    let execution = PipelineExecution {
        results: vec![PipelineStepResult {
            step: PipelineStep::DockerBuild,
            success: false,
            message: "no Dockerfile".to_string(),
            duration_secs: 0.1,
        }],
        overall_success: false,
    };

    let rendered = execution.render();
    assert!(rendered.contains("FAILED"));
    assert!(rendered.contains("✗ Docker Build"));
}

#[test]
fn pipeline_execution_total_duration() {
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

fn make_record(ts: &str, env: &str, success: bool) -> DeploymentRecord {
    DeploymentRecord {
        timestamp: ts.to_string(),
        environment: env.to_string(),
        image_tag: "app:latest".to_string(),
        success,
        steps_completed: 5,
        steps_total: 5,
        duration_secs: 10.0,
        message: "ok".to_string(),
    }
}

#[test]
fn deployment_history_records_and_truncates() {
    let mut history = DeploymentHistory::default();

    for i in 0..60 {
        history.record(make_record(&format!("ts-{i}"), "development", i % 2 == 0));
    }

    assert_eq!(history.total_deployments(), 50);
}

#[test]
fn deployment_history_last_success() {
    let mut history = DeploymentHistory::default();
    history.record(make_record("ts-1", "development", false));
    history.record(make_record("ts-2", "staging", true));

    let last = history.last_success().unwrap();
    assert!(last.success);
    assert_eq!(last.environment, "staging");
}

#[test]
fn deployment_history_yaml_round_trip() {
    let path = std::env::temp_dir().join(format!(
        "kdc-deploy-hist-test-{}.yaml",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let mut history = DeploymentHistory::default();
    history.record(make_record("ts", "development", true));
    history.save(&path).unwrap();

    let loaded = DeploymentHistory::load_or_default(&path).unwrap();
    assert_eq!(history.total_deployments(), loaded.total_deployments());

    std::fs::remove_file(path).unwrap();
}
