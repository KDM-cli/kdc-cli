use crate::domain::project::{ProjectAsset, ProjectAssetKind, ProjectStack};

pub fn detect_stack(assets: &[ProjectAsset]) -> ProjectStack {
    if has(assets, ProjectAssetKind::PackageJson) {
        ProjectStack::Node
    } else if has(assets, ProjectAssetKind::PomXml) {
        ProjectStack::Spring
    } else if has(assets, ProjectAssetKind::CargoToml) {
        ProjectStack::Rust
    } else if has(assets, ProjectAssetKind::GoMod) {
        ProjectStack::Go
    } else if has(assets, ProjectAssetKind::RequirementsTxt) {
        ProjectStack::Python
    } else {
        ProjectStack::Unknown
    }
}

fn has(assets: &[ProjectAsset], kind: ProjectAssetKind) -> bool {
    assets.iter().any(|asset| asset.kind == kind)
}
