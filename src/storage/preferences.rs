use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    pub theme: String,
    pub beginner_mode: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            beginner_mode: true,
        }
    }
}

impl Preferences {
    /// Load preferences from a YAML file, or return defaults.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Unable to read preferences from {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Unable to parse preferences from {}", path.display()))
    }

    /// Save preferences to a YAML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Unable to create preferences directory {}",
                    parent.display()
                )
            })?;
        }

        let content = serde_yaml::to_string(self).context("Unable to serialize preferences")?;
        std::fs::write(path, content)
            .with_context(|| format!("Unable to write preferences to {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_round_trip() {
        let path = std::env::temp_dir().join(format!(
            "kdc-prefs-{}.yaml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let prefs = Preferences {
            theme: "catppuccin".to_string(),
            beginner_mode: false,
        };

        prefs.save(&path).unwrap();
        let loaded = Preferences::load_or_default(&path).unwrap();
        assert_eq!(prefs, loaded);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn preferences_defaults() {
        let prefs = Preferences::default();
        assert_eq!(prefs.theme, "dark");
        assert!(prefs.beginner_mode);
    }
}
