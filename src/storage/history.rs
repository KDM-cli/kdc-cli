use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectHistory {
    pub recent_projects: Vec<PathBuf>,
}

impl ProjectHistory {
    pub fn record_project(&mut self, path: PathBuf) {
        self.recent_projects.retain(|existing| existing != &path);
        self.recent_projects.insert(0, path);
        self.recent_projects.truncate(10);
    }

    pub fn load_or_default(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Unable to read history from {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Unable to parse history from {}", path.display()))
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Unable to create history directory {}", parent.display())
            })?;
        }

        let content = serde_yaml::to_string(self).context("Unable to serialize project history")?;
        std::fs::write(path, content)
            .with_context(|| format!("Unable to write history to {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ProjectHistory;

    #[test]
    fn recent_projects_are_unique_and_newest_first() {
        let mut history = ProjectHistory::default();

        history.record_project(PathBuf::from("/tmp/one"));
        history.record_project(PathBuf::from("/tmp/two"));
        history.record_project(PathBuf::from("/tmp/one"));

        assert_eq!(
            history.recent_projects,
            vec![PathBuf::from("/tmp/one"), PathBuf::from("/tmp/two")]
        );
    }
}
