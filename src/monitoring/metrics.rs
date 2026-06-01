#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceMetrics {
    pub cpu_percent: f32,
    pub memory_percent: f32,
}

impl ResourceMetrics {
    pub fn empty() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_percent: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitoringSnapshot {
    pub health: crate::monitoring::health::HealthReport,
    pub metrics: ResourceMetrics,
    pub logs: Vec<crate::monitoring::logs::LogEntry>,
    pub events: Vec<crate::monitoring::events::MonitoringEvent>,
}

pub fn snapshot(
    runtime: &crate::project::RuntimeCapabilities,
    logs: Vec<crate::monitoring::logs::LogEntry>,
    events: Vec<crate::monitoring::events::MonitoringEvent>,
) -> MonitoringSnapshot {
    MonitoringSnapshot {
        health: crate::monitoring::health::from_runtime(runtime),
        metrics: ResourceMetrics::empty(),
        logs,
        events,
    }
}
