use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A record of a single deployment execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub timestamp: String,
    pub environment: String,
    pub image_tag: String,
    pub success: bool,
    pub steps_completed: usize,
    pub steps_total: usize,
    pub duration_secs: f64,
    pub message: String,
}

/// Persistent deployment history stored as YAML.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeploymentHistory {
    pub records: Vec<DeploymentRecord>,
}

const MAX_RECORDS: usize = 50;

impl DeploymentHistory {
    /// Record a new deployment, keeping only the most recent entries.
    pub fn record(&mut self, entry: DeploymentRecord) {
        self.records.insert(0, entry);
        self.records.truncate(MAX_RECORDS);
    }

    /// Load deployment history from a YAML file, or return an empty history.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Unable to read deploy history from {}", path.display()))?;
        serde_yaml::from_str(&content)
            .with_context(|| format!("Unable to parse deploy history from {}", path.display()))
    }

    /// Save deployment history to a YAML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Unable to create deploy history directory {}",
                    parent.display()
                )
            })?;
        }

        let content =
            serde_yaml::to_string(self).context("Unable to serialize deployment history")?;
        std::fs::write(path, content)
            .with_context(|| format!("Unable to write deploy history to {}", path.display()))
    }

    /// Return the most recent successful deployment, if any.
    pub fn last_success(&self) -> Option<&DeploymentRecord> {
        self.records.iter().find(|r| r.success)
    }

    /// Return the total number of deployments recorded.
    pub fn total_deployments(&self) -> usize {
        self.records.len()
    }

    /// Return the number of successful deployments.
    pub fn successful_deployments(&self) -> usize {
        self.records.iter().filter(|r| r.success).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(success: bool) -> DeploymentRecord {
        DeploymentRecord {
            timestamp: "2026-05-29T12:00:00Z".to_string(),
            environment: "development".to_string(),
            image_tag: "myapp:latest".to_string(),
            success,
            steps_completed: 5,
            steps_total: 5,
            duration_secs: 10.5,
            message: "All steps completed".to_string(),
        }
    }

    #[test]
    fn records_are_newest_first() {
        let mut history = DeploymentHistory::default();
        let mut r1 = sample_record(true);
        r1.timestamp = "2026-05-28T12:00:00Z".to_string();
        let mut r2 = sample_record(true);
        r2.timestamp = "2026-05-29T12:00:00Z".to_string();

        history.record(r1);
        history.record(r2.clone());

        assert_eq!(history.records[0].timestamp, r2.timestamp);
    }

    #[test]
    fn history_truncates_at_max() {
        let mut history = DeploymentHistory::default();
        for i in 0..60 {
            let mut record = sample_record(true);
            record.timestamp = format!("2026-05-29T{i:02}:00:00Z");
            history.record(record);
        }
        assert_eq!(history.records.len(), 50);
    }

    #[test]
    fn last_success_finds_most_recent() {
        let mut history = DeploymentHistory::default();
        history.record(sample_record(false));
        history.record(sample_record(true));

        assert!(history.last_success().is_some());
        assert!(history.last_success().unwrap().success);
    }

    #[test]
    fn counts_are_correct() {
        let mut history = DeploymentHistory::default();
        history.record(sample_record(true));
        history.record(sample_record(false));
        history.record(sample_record(true));

        assert_eq!(history.total_deployments(), 3);
        assert_eq!(history.successful_deployments(), 2);
    }

    #[test]
    fn history_yaml_round_trip() {
        let path = std::env::temp_dir().join(format!(
            "kdc-deploy-history-{}.yaml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let mut history = DeploymentHistory::default();
        history.record(sample_record(true));
        history.save(&path).unwrap();

        let loaded = DeploymentHistory::load_or_default(&path).unwrap();
        assert_eq!(history, loaded);

        std::fs::remove_file(path).unwrap();
    }
}
