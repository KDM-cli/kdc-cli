use std::path::Path;

use anyhow::{Context, Result};

use crate::plugins::registry::{PluginDefinition, PluginRegistry};

pub fn load_installed() -> PluginRegistry {
    load_from_dir(&crate::config::paths::config_dir().join("plugins")).unwrap_or_default()
}

pub fn load_from_dir(path: &Path) -> Result<PluginRegistry> {
    let mut registry = PluginRegistry::default();
    if !path.exists() {
        return Ok(registry);
    }

    for entry in std::fs::read_dir(path)
        .with_context(|| format!("Unable to read plugin directory {}", path.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let is_yaml = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| matches!(extension, "yaml" | "yml"))
            .unwrap_or(false);
        if !is_yaml {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Unable to read plugin manifest {}", path.display()))?;
        let plugin: PluginDefinition = serde_yaml::from_str(&content)
            .with_context(|| format!("Unable to parse plugin manifest {}", path.display()))?;
        registry.register(plugin);
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::load_from_dir;

    #[test]
    fn loads_yaml_plugin_manifests() {
        let dir = std::env::temp_dir().join(format!(
            "kdc-plugins-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("aws.yaml"),
            "name: aws\ncapabilities:\n  - aws\nmenus:\n  - id: aws.eks\n    label: EKS\n",
        )
        .unwrap();

        let registry = load_from_dir(&dir).unwrap();
        assert_eq!(registry.names(), vec!["aws"]);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
