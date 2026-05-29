use std::{fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStack {
    Node,
    Spring,
    Rust,
    Go,
    Python,
    Unknown,
}

impl fmt::Display for ProjectStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ProjectStack::Node => "Node.js",
            ProjectStack::Spring => "Spring Boot",
            ProjectStack::Rust => "Rust",
            ProjectStack::Go => "Go",
            ProjectStack::Python => "Python",
            ProjectStack::Unknown => "Unknown",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAsset {
    pub kind: ProjectAssetKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectAssetKind {
    Dockerfile,
    Compose,
    KubernetesDeployment,
    KubernetesService,
    KubernetesIngress,
    Kustomization,
    HelmChart,
    PackageJson,
    PomXml,
    CargoToml,
    GoMod,
    RequirementsTxt,
}
