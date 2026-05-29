use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
    domain::project::{ProjectAsset, ProjectAssetKind, ProjectStack},
    project::{scanner, stack_detector},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    pub name: String,
    pub root: PathBuf,
    pub stack: ProjectStack,
    pub assets: Vec<ProjectAsset>,
}

impl ProjectContext {
    pub fn has_asset(&self, kind: ProjectAssetKind) -> bool {
        self.assets.iter().any(|asset| asset.kind == kind)
    }

    pub fn has_any_asset(&self, kinds: &[ProjectAssetKind]) -> bool {
        self.assets.iter().any(|asset| kinds.contains(&asset.kind))
    }

    pub fn summary(&self) -> String {
        let assets = scanner::relative_asset_paths(&self.root, &self.assets)
            .into_iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "Project: {}\nRoot: {}\nStack: {}\nAssets: {}",
            self.name,
            self.root.display(),
            self.stack,
            if assets.is_empty() {
                "none".to_string()
            } else {
                assets
            }
        )
    }
}

pub fn detect(path: PathBuf) -> Result<ProjectContext> {
    let root = path
        .canonicalize()
        .with_context(|| format!("Unable to access project path {}", path.display()))?;

    let name = root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    let assets = scanner::scan(&root)?;
    let stack = stack_detector::detect_stack(&assets);

    Ok(ProjectContext {
        name,
        root,
        stack,
        assets,
    })
}
