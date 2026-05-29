use serde::{Deserialize, Serialize};

use crate::{domain::project::ProjectAssetKind, project::detector::ProjectContext};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCapabilities {
    pub docker: bool,
    pub compose: bool,
    pub kubernetes: bool,
    pub helm: bool,
    pub monitoring: bool,
    pub deployment: bool,
    pub templates: bool,
}

impl ProjectCapabilities {
    pub fn summary(&self) -> String {
        format!(
            "Capabilities: docker={}, compose={}, kubernetes={}, helm={}, monitoring={}, deployment={}, templates={}",
            self.docker,
            self.compose,
            self.kubernetes,
            self.helm,
            self.monitoring,
            self.deployment,
            self.templates
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    pub docker_running: bool,
    pub cluster_connected: bool,
    pub registry_connected: bool,
    pub deployment_exists: bool,
    pub rollback_available: bool,
}

pub fn from_project(project: &ProjectContext) -> ProjectCapabilities {
    let docker = project.has_asset(ProjectAssetKind::Dockerfile);
    let compose = project.has_asset(ProjectAssetKind::Compose);
    let kubernetes = project.has_any_asset(&[
        ProjectAssetKind::KubernetesDeployment,
        ProjectAssetKind::KubernetesService,
        ProjectAssetKind::KubernetesIngress,
        ProjectAssetKind::Kustomization,
    ]);
    let helm = project.has_asset(ProjectAssetKind::HelmChart);

    ProjectCapabilities {
        docker,
        compose,
        kubernetes,
        helm,
        monitoring: docker || compose || kubernetes,
        deployment: docker || kubernetes || helm,
        templates: !docker || !kubernetes,
    }
}
