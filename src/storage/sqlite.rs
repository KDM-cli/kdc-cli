use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandHistoryStore {
    pub records: Vec<CommandHistoryRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandHistoryRecord {
    pub action_id: String,
    pub success: bool,
    pub message: String,
}

impl CommandHistoryStore {
    pub fn record(
        &mut self,
        action_id: impl Into<String>,
        success: bool,
        message: impl Into<String>,
    ) {
        self.records.insert(
            0,
            CommandHistoryRecord {
                action_id: action_id.into(),
                success,
                message: message.into(),
            },
        );
        self.records.truncate(100);
    }

    pub fn load_or_default(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Unable to read command history from {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Unable to parse command history from {}", path.display()))
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create {}", parent.display()))?;
        }
        let content = serde_yaml::to_string(self).context("Unable to serialize command history")?;
        std::fs::write(path, content)
            .with_context(|| format!("Unable to write command history to {}", path.display()))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCacheStore {
    pub projects: Vec<ProjectCacheEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCacheEntry {
    pub root: PathBuf,
    pub stack: String,
    pub capabilities: String,
}

impl ProjectCacheStore {
    pub fn upsert(&mut self, entry: ProjectCacheEntry) {
        self.projects.retain(|project| project.root != entry.root);
        self.projects.insert(0, entry);
        self.projects.truncate(50);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CommandHistoryStore, ProjectCacheEntry, ProjectCacheStore};

    #[test]
    fn command_history_is_newest_first() {
        let mut store = CommandHistoryStore::default();
        store.record("a", true, "ok");
        store.record("b", false, "no");
        assert_eq!(store.records[0].action_id, "b");
    }

    #[test]
    fn project_cache_upserts() {
        let mut cache = ProjectCacheStore::default();
        cache.upsert(ProjectCacheEntry {
            root: PathBuf::from("/tmp/app"),
            stack: "Rust".to_string(),
            capabilities: "docker=false".to_string(),
        });
        cache.upsert(ProjectCacheEntry {
            root: PathBuf::from("/tmp/app"),
            stack: "Node.js".to_string(),
            capabilities: "docker=true".to_string(),
        });
        assert_eq!(cache.projects.len(), 1);
        assert_eq!(cache.projects[0].stack, "Node.js");
    }
}
