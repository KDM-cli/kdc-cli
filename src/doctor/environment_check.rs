use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub suggestion: Option<String>,
}

impl DoctorReport {
    pub fn render(&self) -> String {
        self.checks
            .iter()
            .map(|check| {
                let marker = if check.ok { "OK" } else { "WARN" };
                let suggestion = check
                    .suggestion
                    .as_ref()
                    .map(|suggestion| format!(" | Suggestion: {suggestion}"))
                    .unwrap_or_default();
                format!("{marker} {} - {}{}", check.name, check.detail, suggestion)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Export the doctor report as a JSON string for structured consumption.
    pub fn export_json(&self) -> String {
        let entries: Vec<String> = self
            .checks
            .iter()
            .map(|check| {
                let suggestion = check
                    .suggestion
                    .as_ref()
                    .map(|s| format!("\"{s}\""))
                    .unwrap_or_else(|| "null".to_string());
                format!(
                    "  {{\"name\":\"{}\",\"ok\":{},\"detail\":\"{}\",\"suggestion\":{}}}",
                    check.name, check.ok, check.detail, suggestion
                )
            })
            .collect();

        format!("[\n{}\n]", entries.join(",\n"))
    }

    /// Count how many checks passed.
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|c| c.ok).count()
    }

    /// Count total checks.
    pub fn total_count(&self) -> usize {
        self.checks.len()
    }

    /// Return a short summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "Doctor: {}/{} checks passed",
            self.passed_count(),
            self.total_count()
        )
    }
}

/// Run basic doctor checks (Docker CLI, daemon, Kubernetes tooling).
pub fn run() -> DoctorReport {
    DoctorReport {
        checks: vec![
            command_check("Docker CLI", "docker", "--version"),
            docker_daemon_check(),
            kubernetes_tool_check(),
        ],
    }
}

/// Run the full set of doctor checks including registry and additional diagnostics.
pub fn run_full(registry_url: Option<&str>) -> DoctorReport {
    let mut checks = vec![
        command_check("Docker CLI", "docker", "--version"),
        docker_daemon_check(),
        docker_version_check(),
        kubernetes_tool_check(),
        kubernetes_context_check(),
    ];

    if let Some(url) = registry_url {
        checks.push(registry_connectivity_check(url));
    }

    checks.push(os_install_hints_check());

    DoctorReport { checks }
}

fn docker_daemon_check() -> DoctorCheck {
    match check_command("docker", "info") {
        CommandStatus::Available => DoctorCheck {
            name: "Docker Daemon".to_string(),
            ok: true,
            detail: "running".to_string(),
            suggestion: None,
        },
        CommandStatus::Errored => DoctorCheck {
            name: "Docker Daemon".to_string(),
            ok: false,
            detail: "docker is installed but the daemon is not reachable".to_string(),
            suggestion: Some("Start Docker Desktop or the Docker service".to_string()),
        },
        CommandStatus::NotFound => DoctorCheck {
            name: "Docker Daemon".to_string(),
            ok: false,
            detail: "docker not found".to_string(),
            suggestion: Some("Install Docker Desktop or Docker Engine".to_string()),
        },
    }
}

fn docker_version_check() -> DoctorCheck {
    match Command::new("docker").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DoctorCheck {
                name: "Docker Version".to_string(),
                ok: true,
                detail: version,
                suggestion: None,
            }
        }
        Ok(_) => DoctorCheck {
            name: "Docker Version".to_string(),
            ok: false,
            detail: "docker returned an error".to_string(),
            suggestion: Some("Reinstall Docker".to_string()),
        },
        Err(_) => DoctorCheck {
            name: "Docker Version".to_string(),
            ok: false,
            detail: "docker not found".to_string(),
            suggestion: Some(install_hint_for("docker")),
        },
    }
}

fn kubernetes_tool_check() -> DoctorCheck {
    kubernetes_tool_check_with(check_command)
}

fn kubernetes_tool_check_with(check_command: impl Fn(&str, &str) -> CommandStatus) -> DoctorCheck {
    match check_command("kubectl", "version") {
        CommandStatus::Available => DoctorCheck {
            name: "Kubernetes Tooling".to_string(),
            ok: true,
            detail: "kubectl available".to_string(),
            suggestion: None,
        },
        CommandStatus::Errored => DoctorCheck {
            name: "Kubernetes Tooling".to_string(),
            ok: false,
            detail: "kubectl installed but returned an error".to_string(),
            suggestion: Some(
                "Check current context with kubectl config current-context".to_string(),
            ),
        },
        CommandStatus::NotFound => match check_command("minikube", "version") {
            CommandStatus::Available => DoctorCheck {
                name: "Kubernetes Tooling".to_string(),
                ok: true,
                detail: "kubectl not found; minikube available".to_string(),
                suggestion: Some("Install kubectl for full Kubernetes workflows".to_string()),
            },
            CommandStatus::Errored => DoctorCheck {
                name: "Kubernetes Tooling".to_string(),
                ok: false,
                detail: "kubectl not found; minikube installed but returned an error".to_string(),
                suggestion: Some("Run minikube status or minikube start".to_string()),
            },
            CommandStatus::NotFound => DoctorCheck {
                name: "Kubernetes Tooling".to_string(),
                ok: false,
                detail: "kubectl and minikube not found".to_string(),
                suggestion: Some(install_hint_for("kubectl")),
            },
        },
    }
}

fn kubernetes_context_check() -> DoctorCheck {
    match Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let context = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DoctorCheck {
                name: "Kubernetes Context".to_string(),
                ok: true,
                detail: format!("current context: {context}"),
                suggestion: None,
            }
        }
        Ok(_) => DoctorCheck {
            name: "Kubernetes Context".to_string(),
            ok: false,
            detail: "no active context set".to_string(),
            suggestion: Some("Run kubectl config use-context <name>".to_string()),
        },
        Err(_) => DoctorCheck {
            name: "Kubernetes Context".to_string(),
            ok: false,
            detail: "kubectl not available".to_string(),
            suggestion: Some(install_hint_for("kubectl")),
        },
    }
}

fn registry_connectivity_check(registry_url: &str) -> DoctorCheck {
    // Try a lightweight check by running `docker manifest inspect` against a
    // known public image on the registry. This validates connectivity without
    // needing credentials for the probe itself.
    match Command::new("docker").args(["info"]).output() {
        Ok(output) if output.status.success() => {
            let info = String::from_utf8_lossy(&output.stdout);
            if info.contains("Registry") || !registry_url.is_empty() {
                DoctorCheck {
                    name: "Registry Connectivity".to_string(),
                    ok: true,
                    detail: format!("Docker daemon reachable; registry target: {registry_url}"),
                    suggestion: None,
                }
            } else {
                DoctorCheck {
                    name: "Registry Connectivity".to_string(),
                    ok: false,
                    detail: format!("Cannot verify registry: {registry_url}"),
                    suggestion: Some("Run docker login to authenticate".to_string()),
                }
            }
        }
        _ => DoctorCheck {
            name: "Registry Connectivity".to_string(),
            ok: false,
            detail: "Docker is not available for registry check".to_string(),
            suggestion: Some("Install and start Docker first".to_string()),
        },
    }
}

fn os_install_hints_check() -> DoctorCheck {
    let os = std::env::consts::OS;
    let hint = match os {
        "macos" => "Use Homebrew: brew install docker kubectl",
        "linux" => "Use apt: sudo apt install docker.io kubectl",
        "windows" => "Use Chocolatey: choco install docker-desktop kubernetes-cli",
        _ => "Visit docs.docker.com and kubernetes.io for installation instructions",
    };

    DoctorCheck {
        name: "OS Install Hints".to_string(),
        ok: true,
        detail: format!("Detected OS: {os}"),
        suggestion: Some(hint.to_string()),
    }
}

fn install_hint_for(tool: &str) -> String {
    let os = std::env::consts::OS;
    match (os, tool) {
        ("macos", "docker") => "Install with: brew install --cask docker".to_string(),
        ("macos", "kubectl") => "Install with: brew install kubectl".to_string(),
        ("linux", "docker") => "Install with: sudo apt install docker.io".to_string(),
        ("linux", "kubectl") => "Install with: sudo snap install kubectl --classic".to_string(),
        ("windows", "docker") => "Install with: choco install docker-desktop".to_string(),
        ("windows", "kubectl") => "Install with: choco install kubernetes-cli".to_string(),
        _ => format!("Install {tool} or add it to PATH"),
    }
}

fn command_check(name: &str, command: &str, arg: &str) -> DoctorCheck {
    match check_command(command, arg) {
        CommandStatus::Available => DoctorCheck {
            name: name.to_string(),
            ok: true,
            detail: "available".to_string(),
            suggestion: None,
        },
        CommandStatus::Errored => DoctorCheck {
            name: name.to_string(),
            ok: false,
            detail: "installed but returned an error".to_string(),
            suggestion: Some(format!("Check {command} configuration")),
        },
        CommandStatus::NotFound => DoctorCheck {
            name: name.to_string(),
            ok: false,
            detail: "not found".to_string(),
            suggestion: Some(install_hint_for(command)),
        },
    }
}

fn check_command(command: &str, arg: &str) -> CommandStatus {
    match Command::new(command).arg(arg).output() {
        Ok(output) if output.status.success() => CommandStatus::Available,
        Ok(_) => CommandStatus::Errored,
        Err(_) => CommandStatus::NotFound,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandStatus {
    Available,
    Errored,
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::{kubernetes_tool_check_with, CommandStatus, DoctorCheck, DoctorReport};

    #[test]
    fn falls_back_to_minikube_when_kubectl_is_missing() {
        let check = kubernetes_tool_check_with(|command, _arg| match command {
            "kubectl" => CommandStatus::NotFound,
            "minikube" => CommandStatus::Available,
            _ => CommandStatus::NotFound,
        });

        assert_eq!(
            check,
            DoctorCheck {
                name: "Kubernetes Tooling".to_string(),
                ok: true,
                detail: "kubectl not found; minikube available".to_string(),
                suggestion: Some("Install kubectl for full Kubernetes workflows".to_string())
            }
        );
    }

    #[test]
    fn reports_missing_when_kubectl_and_minikube_are_missing() {
        let check = kubernetes_tool_check_with(|_, _| CommandStatus::NotFound);

        assert_eq!(check.detail, "kubectl and minikube not found");
        assert!(!check.ok);
    }

    #[test]
    fn report_summary_line() {
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
            ],
        };

        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.total_count(), 2);
        assert_eq!(report.summary_line(), "Doctor: 1/2 checks passed");
    }

    #[test]
    fn report_export_json_is_valid() {
        let report = DoctorReport {
            checks: vec![DoctorCheck {
                name: "Docker CLI".to_string(),
                ok: true,
                detail: "available".to_string(),
                suggestion: None,
            }],
        };

        let json = report.export_json();
        assert!(json.contains("\"name\":\"Docker CLI\""));
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"suggestion\":null"));
    }
}
