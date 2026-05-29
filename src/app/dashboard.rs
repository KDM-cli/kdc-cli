use crate::app::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardCard {
    pub title: String,
    pub value: String,
}

pub fn cards(state: &AppState) -> Vec<DashboardCard> {
    let mut cards = vec![
        DashboardCard {
            title: "Project".to_string(),
            value: state.project.name.clone(),
        },
        DashboardCard {
            title: "Stack".to_string(),
            value: state.project.stack.to_string(),
        },
    ];

    if state.capabilities.docker {
        cards.push(DashboardCard {
            title: "Docker".to_string(),
            value: "Detected".to_string(),
        });
    }

    if state.capabilities.kubernetes {
        cards.push(DashboardCard {
            title: "Kubernetes".to_string(),
            value: "Manifests".to_string(),
        });
    }

    cards
}
