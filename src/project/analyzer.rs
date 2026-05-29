use crate::{
    domain::project::ProjectAssetKind,
    project::{scanner, ProjectCapabilities, ProjectContext, RuntimeCapabilities},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAnalysis {
    pub project_name: String,
    pub root: String,
    pub stack: String,
    pub capabilities: ProjectCapabilities,
    pub runtime: RuntimeCapabilities,
    pub detected_assets: Vec<String>,
    pub missing_recommended_assets: Vec<String>,
    pub next_steps: Vec<String>,
}

impl ProjectAnalysis {
    pub fn from_context(
        project: &ProjectContext,
        capabilities: ProjectCapabilities,
        runtime: RuntimeCapabilities,
    ) -> Self {
        let detected_assets = scanner::relative_asset_paths(&project.root, &project.assets)
            .into_iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let missing_recommended_assets = missing_assets(project, &capabilities);
        let next_steps = next_steps(&capabilities, &runtime);

        Self {
            project_name: project.name.clone(),
            root: project.root.display().to_string(),
            stack: project.stack.to_string(),
            capabilities,
            runtime,
            detected_assets,
            missing_recommended_assets,
            next_steps,
        }
    }

    pub fn render(&self) -> String {
        let detected_assets = render_list(&self.detected_assets, "none");
        let missing_assets = render_list(&self.missing_recommended_assets, "none");
        let next_steps = render_list(&self.next_steps, "none");

        format!(
            "Project Analysis\n\
             Project: {}\n\
             Root: {}\n\
             Stack: {}\n\
             Docker: {}\n\
             Compose: {}\n\
             Kubernetes: {}\n\
             Helm: {}\n\
             Docker Runtime: {}\n\
             Cluster Runtime: {}\n\
             Detected Assets:\n{}\n\
             Missing Recommended Assets:\n{}\n\
             Next Steps:\n{}",
            self.project_name,
            self.root,
            self.stack,
            enabled(self.capabilities.docker),
            enabled(self.capabilities.compose),
            enabled(self.capabilities.kubernetes),
            enabled(self.capabilities.helm),
            running(self.runtime.docker_running),
            running(self.runtime.cluster_connected),
            detected_assets,
            missing_assets,
            next_steps
        )
    }
}

fn missing_assets(project: &ProjectContext, capabilities: &ProjectCapabilities) -> Vec<String> {
    let mut missing = Vec::new();

    if !capabilities.docker {
        missing.push("Dockerfile".to_string());
    }

    if !capabilities.compose {
        missing.push("docker-compose.yml or compose.yaml".to_string());
    }

    if !capabilities.kubernetes {
        missing.push("deployment.yaml and service.yaml".to_string());
    } else {
        if !project.has_asset(ProjectAssetKind::KubernetesDeployment) {
            missing.push("deployment.yaml".to_string());
        }
        if !project.has_asset(ProjectAssetKind::KubernetesService) {
            missing.push("service.yaml".to_string());
        }
    }

    missing
}

fn next_steps(capabilities: &ProjectCapabilities, runtime: &RuntimeCapabilities) -> Vec<String> {
    let mut steps = Vec::new();

    if capabilities.docker && !runtime.docker_running {
        steps.push("Start Docker before running container actions".to_string());
    }
    if capabilities.kubernetes && !runtime.cluster_connected {
        steps.push("Connect Kubernetes or start Minikube before deployment actions".to_string());
    }
    if !capabilities.docker {
        steps.push("Generate or add a Dockerfile".to_string());
    }
    if !capabilities.kubernetes {
        steps.push("Generate or add Kubernetes manifests".to_string());
    }
    if capabilities.docker && capabilities.kubernetes {
        steps.push("Use the action list to build, deploy, and monitor the application".to_string());
    }

    steps
}

fn enabled(value: bool) -> &'static str {
    if value {
        "enabled"
    } else {
        "not configured"
    }
}

fn running(value: bool) -> &'static str {
    if value {
        "available"
    } else {
        "unavailable"
    }
}

fn render_list(values: &[String], empty: &str) -> String {
    if values.is_empty() {
        format!("  - {empty}")
    } else {
        values
            .iter()
            .map(|value| format!("  - {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
