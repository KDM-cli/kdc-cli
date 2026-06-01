use kdc::doctor::{
    docker_check::DockerStatus,
    environment_check::{DoctorCheck, DoctorReport},
    kubernetes_check::KubernetesStatus,
    registry_check::{RegistryConfig, RegistryStatus},
};

#[test]
fn doctor_report_export_json() {
    let report = DoctorReport {
        checks: vec![
            DoctorCheck {
                name: "Docker CLI".to_string(),
                ok: true,
                detail: "available".to_string(),
                suggestion: None,
            },
            DoctorCheck {
                name: "Docker Daemon".to_string(),
                ok: false,
                detail: "not reachable".to_string(),
                suggestion: Some("Start Docker Desktop".to_string()),
            },
        ],
    };

    let json = report.export_json();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["checks"][0]["name"], "Docker CLI");
    assert_eq!(val["checks"][0]["ok"], true);
    assert_eq!(val["checks"][0]["suggestion"], serde_json::Value::Null);
    assert_eq!(val["checks"][1]["name"], "Docker Daemon");
    assert_eq!(val["checks"][1]["ok"], false);
    assert_eq!(val["checks"][1]["suggestion"], "Start Docker Desktop");
}

#[test]
fn doctor_report_summary_line() {
    let report = DoctorReport {
        checks: vec![
            DoctorCheck {
                name: "A".to_string(),
                ok: true,
                detail: "ok".to_string(),
                suggestion: None,
            },
            DoctorCheck {
                name: "B".to_string(),
                ok: false,
                detail: "fail".to_string(),
                suggestion: Some("fix".to_string()),
            },
            DoctorCheck {
                name: "C".to_string(),
                ok: true,
                detail: "ok".to_string(),
                suggestion: None,
            },
        ],
    };

    assert_eq!(report.passed_count(), 2);
    assert_eq!(report.total_count(), 3);
    assert_eq!(report.summary_line(), "Doctor: 2/3 checks passed");
}

#[test]
fn doctor_report_render() {
    let report = DoctorReport {
        checks: vec![DoctorCheck {
            name: "Docker CLI".to_string(),
            ok: true,
            detail: "available".to_string(),
            suggestion: None,
        }],
    };

    let rendered = report.render();
    assert!(rendered.contains("OK Docker CLI - available"));
}

#[test]
fn doctor_report_warning_render() {
    let report = DoctorReport {
        checks: vec![DoctorCheck {
            name: "Docker Daemon".to_string(),
            ok: false,
            detail: "not running".to_string(),
            suggestion: Some("Start Docker Desktop".to_string()),
        }],
    };

    let rendered = report.render();
    assert!(rendered.contains("WARN Docker Daemon"));
    assert!(rendered.contains("Suggestion: Start Docker Desktop"));
}

#[test]
fn docker_status_enum_values() {
    assert_ne!(DockerStatus::Running, DockerStatus::Unavailable);
    assert_ne!(DockerStatus::Unknown, DockerStatus::Running);
    assert_ne!(DockerStatus::Unknown, DockerStatus::Unavailable);
}

#[test]
fn kubernetes_status_enum_values() {
    assert_ne!(KubernetesStatus::Connected, KubernetesStatus::Disconnected);
    assert_ne!(KubernetesStatus::Unknown, KubernetesStatus::Connected);
}

#[test]
fn registry_status_enum_values() {
    assert_ne!(RegistryStatus::Connected, RegistryStatus::Disconnected);
    assert_ne!(RegistryStatus::Unknown, RegistryStatus::Connected);
}

#[test]
fn registry_config_defaults() {
    let config = RegistryConfig::default();
    assert_eq!(config.url, "docker.io");
    assert!(config.username.is_none());
}

#[test]
fn registry_config_custom() {
    let config = RegistryConfig {
        url: "ghcr.io".to_string(),
        username: Some("user".to_string()),
    };
    assert_eq!(config.url, "ghcr.io");
    assert_eq!(config.username, Some("user".to_string()));
}
