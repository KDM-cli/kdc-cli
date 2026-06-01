#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub docker: HealthStatus,
    pub kubernetes: HealthStatus,
    pub registry: HealthStatus,
}

impl HealthReport {
    pub fn overall(&self) -> HealthStatus {
        if [self.docker, self.kubernetes, self.registry].contains(&HealthStatus::Critical) {
            HealthStatus::Critical
        } else if [self.docker, self.kubernetes, self.registry].contains(&HealthStatus::Warning) {
            HealthStatus::Warning
        } else if [self.docker, self.kubernetes, self.registry].contains(&HealthStatus::Unknown) {
            HealthStatus::Unknown
        } else {
            HealthStatus::Healthy
        }
    }
}

pub fn from_runtime(runtime: &crate::project::RuntimeCapabilities) -> HealthReport {
    HealthReport {
        docker: if runtime.docker_running {
            HealthStatus::Healthy
        } else {
            HealthStatus::Warning
        },
        kubernetes: if runtime.cluster_connected {
            HealthStatus::Healthy
        } else {
            HealthStatus::Warning
        },
        registry: if runtime.registry_connected {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::project::RuntimeCapabilities;

    use super::{from_runtime, HealthStatus};

    #[test]
    fn health_report_rolls_up_warning() {
        let report = from_runtime(&RuntimeCapabilities::default());
        assert_eq!(report.overall(), HealthStatus::Warning);
    }
}
