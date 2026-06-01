#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub source: String,
    pub message: String,
}

pub fn docker_logs(tail: u16) -> anyhow::Result<Vec<LogEntry>> {
    let containers = crate::docker::containers::list()?;
    let Some(container) = containers.first() else {
        return Ok(Vec::new());
    };
    let lines = crate::docker::logs::fetch(&container.id, tail.into())?;
    Ok(lines
        .into_iter()
        .map(|line| LogEntry {
            source: format!("docker/{}", container.name),
            message: line.message,
        })
        .collect())
}

pub fn compose_logs(root: &std::path::Path, tail: u16) -> anyhow::Result<Vec<LogEntry>> {
    let request = crate::compose::logs::ComposeLogRequest {
        tail: Some(tail.into()),
        ..Default::default()
    };
    Ok(crate::compose::logs::fetch(&request, root)?
        .into_iter()
        .map(|message| LogEntry {
            source: "compose".to_string(),
            message,
        })
        .collect())
}

pub fn pod_logs(namespace: &str, tail: u16) -> anyhow::Result<Vec<LogEntry>> {
    let pods = crate::kubernetes::pods::list(namespace)?;
    let Some(pod) = pods.first() else {
        return Ok(Vec::new());
    };
    Ok(
        crate::kubernetes::pods::logs(&pod.name, namespace, Some(tail))?
            .into_iter()
            .map(|message| LogEntry {
                source: format!("pod/{}", pod.name),
                message,
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::LogEntry;

    #[test]
    fn log_entry_fields() {
        let entry = LogEntry {
            source: "app".to_string(),
            message: "ready".to_string(),
        };
        assert_eq!(entry.source, "app");
    }
}
