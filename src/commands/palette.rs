use crate::{
    domain::screen::Screen,
    project::{ProjectCapabilities, ProjectContext, RuntimeCapabilities},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAction {
    pub id: String,
    pub label: String,
    pub screen: Screen,
    pub enabled: bool,
    pub reason: Option<String>,
    pub shortcut: Option<String>,
}

impl CommandAction {
    pub fn enabled(id: &str, label: &str, screen: Screen, shortcut: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            screen,
            enabled: true,
            reason: None,
            shortcut: shortcut.map(str::to_string),
        }
    }

    pub fn disabled(id: &str, label: &str, screen: Screen, reason: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            screen,
            enabled: false,
            reason: Some(reason.to_string()),
            shortcut: None,
        }
    }

    pub fn display_line(&self) -> String {
        let state = if self.enabled { "enabled" } else { "disabled" };
        let shortcut = self
            .shortcut
            .as_ref()
            .map(|shortcut| format!(" [{shortcut}]"))
            .unwrap_or_default();
        let reason = self
            .reason
            .as_ref()
            .map(|reason| format!(" - {reason}"))
            .unwrap_or_default();

        format!("{}{} - {}{}", self.label, shortcut, state, reason)
    }
}

pub fn generate_actions(
    project: &ProjectContext,
    capabilities: &ProjectCapabilities,
    runtime: &RuntimeCapabilities,
) -> Vec<CommandAction> {
    let mut actions = vec![
        CommandAction::enabled(
            "project.analysis",
            "Project Analysis",
            Screen::Dashboard,
            None,
        ),
        CommandAction::enabled(
            "project.refresh",
            "Refresh Project Scan",
            Screen::Dashboard,
            None,
        ),
    ];

    if capabilities.docker {
        actions.push(CommandAction::enabled(
            "docker.build",
            "Build Docker Image",
            Screen::Docker,
            Some("Ctrl+B"),
        ));
        actions.push(if runtime.docker_running {
            CommandAction::enabled("docker.run", "Run Container", Screen::Docker, None)
        } else {
            CommandAction::disabled(
                "docker.run",
                "Run Container",
                Screen::Docker,
                "Docker daemon is not running",
            )
        });
        actions.push(CommandAction::enabled(
            "docker.logs",
            "Docker Logs",
            Screen::Monitoring,
            Some("Ctrl+L"),
        ));
    }

    if capabilities.compose {
        actions.push(CommandAction::enabled(
            "compose.up",
            "Compose Up",
            Screen::Compose,
            None,
        ));
        actions.push(CommandAction::enabled(
            "compose.down",
            "Compose Down",
            Screen::Compose,
            None,
        ));
        actions.push(CommandAction::enabled(
            "compose.logs",
            "Compose Logs",
            Screen::Monitoring,
            None,
        ));
    }

    if capabilities.kubernetes {
        actions.push(if runtime.cluster_connected {
            CommandAction::enabled(
                "kubernetes.deploy",
                "Deploy Application",
                Screen::Deployments,
                Some("Ctrl+D"),
            )
        } else {
            CommandAction::disabled(
                "kubernetes.deploy",
                "Deploy Application",
                Screen::Deployments,
                "Kubernetes cluster is not connected",
            )
        });
        actions.push(CommandAction::enabled(
            "kubernetes.pods",
            "View Pods",
            Screen::Kubernetes,
            None,
        ));
        actions.push(CommandAction::enabled(
            "kubernetes.services",
            "View Services",
            Screen::Kubernetes,
            None,
        ));
        actions.push(if runtime.rollback_available {
            CommandAction::enabled(
                "kubernetes.rollback",
                "Rollback Deployment",
                Screen::Deployments,
                None,
            )
        } else {
            CommandAction::disabled(
                "kubernetes.rollback",
                "Rollback Deployment",
                Screen::Deployments,
                "No rollback history detected",
            )
        });
    }

    if capabilities.helm {
        actions.push(CommandAction::enabled(
            "helm.upgrade",
            "Helm Upgrade",
            Screen::Helm,
            None,
        ));
    }

    if capabilities.templates {
        actions.push(CommandAction::enabled(
            "templates.generate",
            "Generate Missing Project Templates",
            Screen::Settings,
            None,
        ));
    }

    actions.push(CommandAction::enabled(
        "settings.open",
        "Open Project Settings",
        Screen::Settings,
        None,
    ));

    actions.sort_by(|left, right| left.label.cmp(&right.label));

    if project.stack.to_string() != "Unknown" {
        actions.push(CommandAction::enabled(
            "stack.build",
            &format!("Build {} Project", project.stack),
            Screen::Dashboard,
            None,
        ));
    }

    actions
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        domain::project::ProjectStack,
        project::{ProjectCapabilities, ProjectContext, RuntimeCapabilities},
    };

    use super::generate_actions;

    #[test]
    fn kubernetes_deploy_is_disabled_without_cluster() {
        let project = ProjectContext {
            name: "app".to_string(),
            root: PathBuf::from("."),
            stack: ProjectStack::Rust,
            assets: Vec::new(),
        };
        let actions = generate_actions(
            &project,
            &ProjectCapabilities {
                kubernetes: true,
                deployment: true,
                ..ProjectCapabilities::default()
            },
            &RuntimeCapabilities::default(),
        );

        let deploy = actions
            .iter()
            .find(|action| action.id == "kubernetes.deploy")
            .unwrap();

        assert!(!deploy.enabled);
        assert_eq!(
            deploy.reason.as_deref(),
            Some("Kubernetes cluster is not connected")
        );
    }
}
