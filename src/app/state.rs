use crate::{
    commands::palette::CommandAction,
    config::settings::Settings,
    domain::{menu::MenuItem, screen::Screen},
    project::{ProjectCapabilities, ProjectContext, RuntimeCapabilities},
    storage::history::ProjectHistory,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub project: ProjectContext,
    pub capabilities: ProjectCapabilities,
    pub runtime: RuntimeCapabilities,
    pub current_screen: Screen,
    pub selected_menu: usize,
    pub menus: Vec<MenuItem>,
    pub actions: Vec<CommandAction>,
    pub settings: Settings,
    pub history: ProjectHistory,
    pub status_message: String,
}

impl AppState {
    pub fn new(
        project: ProjectContext,
        capabilities: ProjectCapabilities,
        runtime: RuntimeCapabilities,
        menus: Vec<MenuItem>,
        actions: Vec<CommandAction>,
        settings: Settings,
        history: ProjectHistory,
    ) -> Self {
        Self {
            status_message: format!("Loaded {}", project.name),
            project,
            capabilities,
            runtime,
            current_screen: Screen::Dashboard,
            selected_menu: 0,
            menus,
            actions,
            settings,
            history,
        }
    }

    pub fn selected_menu(&self) -> Option<&MenuItem> {
        self.menus.get(self.selected_menu)
    }
}
