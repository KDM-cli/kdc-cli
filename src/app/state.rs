use crate::{
    commands::palette::CommandAction,
    config::settings::Settings,
    domain::{menu::MenuItem, screen::Screen},
    project::{runtime, ProjectCapabilities, ProjectContext, RuntimeCapabilities},
    storage::history::ProjectHistory,
    ui::{
        menus::CapabilityMenuGenerator,
        state::{Notification, UiState},
        theme::ThemeName,
    },
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
    pub ui: UiState,
    pub status_message: String,
}

impl AppState {
    pub fn new(init: AppStateInit) -> Self {
        let active_theme = ThemeName::from_setting(&init.settings.theme);
        Self {
            status_message: format!("Loaded {}", init.project.name),
            project: init.project,
            capabilities: init.capabilities,
            runtime: init.runtime,
            current_screen: Screen::Dashboard,
            selected_menu: 0,
            menus: init.menus,
            actions: init.actions,
            settings: init.settings,
            history: init.history,
            ui: UiState::new(init.is_first_launch, active_theme),
        }
    }

    pub fn selected_menu(&self) -> Option<&MenuItem> {
        self.menus.get(self.selected_menu)
    }

    pub fn notify_info(&mut self, message: impl Into<String>) {
        self.ui
            .push_notification(Notification::info(message.into()));
    }

    pub fn notify_warning(&mut self, message: impl Into<String>) {
        self.ui
            .push_notification(Notification::warning(message.into()));
    }

    pub fn refresh_runtime(&mut self) {
        self.runtime = runtime::detect(&self.capabilities);
        self.menus = CapabilityMenuGenerator::generate(&self.capabilities, &self.runtime);
        self.actions = crate::commands::palette::generate_actions(
            &self.project,
            &self.capabilities,
            &self.runtime,
        );
        self.status_message = "Runtime refreshed".to_string();
    }
}

#[derive(Debug, Clone)]
pub struct AppStateInit {
    pub project: ProjectContext,
    pub capabilities: ProjectCapabilities,
    pub runtime: RuntimeCapabilities,
    pub menus: Vec<MenuItem>,
    pub actions: Vec<CommandAction>,
    pub settings: Settings,
    pub history: ProjectHistory,
    pub is_first_launch: bool,
}
