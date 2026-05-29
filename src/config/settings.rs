use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub default_environment: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            default_environment: "development".to_string(),
        }
    }
}

impl Settings {
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Unable to read settings from {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Unable to parse settings from {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Unable to create config directory {}", parent.display())
            })?;
        }

        let content = serde_yaml::to_string(self).context("Unable to serialize settings")?;
        fs::write(path, content)
            .with_context(|| format!("Unable to write settings to {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::Settings;

    #[test]
    fn settings_round_trip_as_yaml() {
        let path = std::env::temp_dir().join(format!(
            "kdc-settings-{}.yaml",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let settings = Settings {
            theme: "nord".to_string(),
            default_environment: "staging".to_string(),
        };

        settings.save(&path).unwrap();
        let loaded = Settings::load_or_default(&path).unwrap();

        assert_eq!(settings, loaded);
        fs::remove_file(path).unwrap();
    }
}
