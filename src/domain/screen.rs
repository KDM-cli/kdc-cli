#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Docker,
    Compose,
    Kubernetes,
    Helm,
    Deployments,
    Monitoring,
    Settings,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Docker => "Docker",
            Screen::Compose => "Compose",
            Screen::Kubernetes => "Kubernetes",
            Screen::Helm => "Helm",
            Screen::Deployments => "Deployments",
            Screen::Monitoring => "Monitoring",
            Screen::Settings => "Settings",
        }
    }
}
