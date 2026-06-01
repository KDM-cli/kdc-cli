use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::domain::project::{ProjectAsset, ProjectAssetKind};

const MAX_SCAN_DEPTH: usize = 5;

pub fn scan(root: &Path) -> Result<Vec<ProjectAsset>> {
    let mut assets = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(MAX_SCAN_DEPTH)
        .into_iter()
        .filter_entry(|entry| !is_ignored(entry.path()))
    {
        // Skip entries that fail with permission errors instead of failing the entire scan
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                // Log the error but continue scanning
                tracing::debug!("Skipping inaccessible path: {}", err);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        if let Some(kind) = classify(entry.path()) {
            assets.push(ProjectAsset {
                kind,
                path: entry.path().to_path_buf(),
            });
        }
    }

    assets.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(assets)
}

pub fn classify(path: &Path) -> Option<ProjectAssetKind> {
    let filename = path.file_name()?.to_string_lossy();

    match filename.as_ref() {
        "Dockerfile" => Some(ProjectAssetKind::Dockerfile),
        "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml" => {
            Some(ProjectAssetKind::Compose)
        }
        "deployment.yaml" | "deployment.yml" => Some(ProjectAssetKind::KubernetesDeployment),
        "service.yaml" | "service.yml" => Some(ProjectAssetKind::KubernetesService),
        "ingress.yaml" | "ingress.yml" => Some(ProjectAssetKind::KubernetesIngress),
        "kustomization.yaml" | "kustomization.yml" => Some(ProjectAssetKind::Kustomization),
        "Chart.yaml" => Some(ProjectAssetKind::HelmChart),
        "package.json" => Some(ProjectAssetKind::PackageJson),
        "pom.xml" => Some(ProjectAssetKind::PomXml),
        "Cargo.toml" => Some(ProjectAssetKind::CargoToml),
        "go.mod" => Some(ProjectAssetKind::GoMod),
        "requirements.txt" => Some(ProjectAssetKind::RequirementsTxt),
        _ => None,
    }
}

fn is_ignored(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            ".git" | "target" | "node_modules" | ".venv" | "dist" | "build"
        )
    })
}

pub fn relative_asset_paths(root: &Path, assets: &[ProjectAsset]) -> Vec<PathBuf> {
    assets
        .iter()
        .map(|asset| {
            asset
                .path
                .strip_prefix(root)
                .unwrap_or(&asset.path)
                .to_path_buf()
        })
        .collect()
}
