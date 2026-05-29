use crate::{
    domain::{menu::MenuItem, screen::Screen},
    project::{ProjectCapabilities, RuntimeCapabilities},
};

pub struct CapabilityMenuGenerator;

impl CapabilityMenuGenerator {
    pub fn generate(
        capabilities: &ProjectCapabilities,
        runtime: &RuntimeCapabilities,
    ) -> Vec<MenuItem> {
        let mut menus = vec![MenuItem::visible(
            "dashboard",
            "Dashboard",
            Screen::Dashboard,
            None,
        )];

        if capabilities.docker {
            menus.push(MenuItem::visible(
                "docker",
                "Docker",
                Screen::Docker,
                Some("Ctrl+B"),
            ));
        }

        if capabilities.compose {
            menus.push(MenuItem::visible(
                "compose",
                "Compose",
                Screen::Compose,
                None,
            ));
        }

        if capabilities.kubernetes {
            menus.push(MenuItem::visible(
                "kubernetes",
                "Kubernetes",
                Screen::Kubernetes,
                None,
            ));
            menus.push(if runtime.cluster_connected {
                MenuItem::visible(
                    "deployments",
                    "Deployments",
                    Screen::Deployments,
                    Some("Ctrl+D"),
                )
            } else {
                MenuItem::disabled("deployments", "Deployments", Screen::Deployments)
            });
        }

        if capabilities.helm {
            menus.push(MenuItem::visible("helm", "Helm", Screen::Helm, None));
        }

        if capabilities.monitoring {
            menus.push(MenuItem::visible(
                "monitoring",
                "Monitoring",
                Screen::Monitoring,
                Some("Ctrl+L"),
            ));
        }

        menus.push(MenuItem::visible(
            "settings",
            "Settings",
            Screen::Settings,
            None,
        ));

        menus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_infrastructure_menus_without_capabilities() {
        let menus = CapabilityMenuGenerator::generate(
            &ProjectCapabilities::default(),
            &RuntimeCapabilities::default(),
        );

        let labels = menus
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["Dashboard", "Settings"]);
    }

    #[test]
    fn shows_docker_compose_and_kubernetes_when_supported() {
        let capabilities = ProjectCapabilities {
            docker: true,
            compose: true,
            kubernetes: true,
            monitoring: true,
            deployment: true,
            ..ProjectCapabilities::default()
        };

        let menus = CapabilityMenuGenerator::generate(
            &capabilities,
            &RuntimeCapabilities {
                cluster_connected: true,
                ..RuntimeCapabilities::default()
            },
        );

        let labels = menus
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"Docker"));
        assert!(labels.contains(&"Compose"));
        assert!(labels.contains(&"Kubernetes"));
        assert!(labels.contains(&"Deployments"));
        assert!(labels.contains(&"Monitoring"));
    }
}
