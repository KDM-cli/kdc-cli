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
}

pub fn run() -> DoctorReport {
    DoctorReport {
        checks: vec![
            command_check("Docker CLI", "docker", "--version"),
            docker_daemon_check(),
            kubernetes_tool_check(),
        ],
    }
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
                suggestion: Some("Install kubectl or Minikube".to_string()),
            },
        },
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
            suggestion: Some(format!("Install {command} or add it to PATH")),
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
    use super::{kubernetes_tool_check_with, CommandStatus, DoctorCheck};

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
}
