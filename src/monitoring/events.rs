#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitoringEvent {
    pub message: String,
}

pub fn kubernetes_events(namespace: &str) -> anyhow::Result<Vec<MonitoringEvent>> {
    Ok(crate::kubernetes::events::list(namespace)?
        .into_iter()
        .map(|event| MonitoringEvent {
            message: format!("{}: {}", event.reason, event.message),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::MonitoringEvent;

    #[test]
    fn event_holds_message() {
        let event = MonitoringEvent {
            message: "rolled out".to_string(),
        };
        assert_eq!(event.message, "rolled out");
    }
}
