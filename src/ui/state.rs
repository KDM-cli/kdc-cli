use std::path::PathBuf;

use crate::{commands::palette::CommandAction, domain::screen::Screen, ui::theme::ThemeName};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiPhase {
    FirstLaunch,
    Scanning,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstLaunchChoice {
    UseCurrentFolder,
    BrowseFolder,
    Exit,
}

impl FirstLaunchChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::UseCurrentFolder => "Initialize KDC in current directory",
            Self::BrowseFolder => "Select another directory",
            Self::Exit => "Exit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Sidebar,
    Main,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub level: NotificationLevel,
    pub message: String,
    pub ttl: u8,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Info,
            message: message.into(),
            ttl: 8,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Success,
            message: message.into(),
            ttl: 8,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: NotificationLevel::Warning,
            message: message.into(),
            ttl: 10,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.ttl = self.ttl.saturating_sub(1);
        self.ttl > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
}

impl CommandPaletteState {
    pub fn closed() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected = 0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiState {
    pub phase: UiPhase,
    pub first_launch_choice: usize,
    pub scan_progress: u16,
    pub palette: CommandPaletteState,
    pub notifications: Vec<Notification>,
    pub active_theme: ThemeName,
    pub picked_folder: Option<PathBuf>,
    pub execution_output: Option<Vec<String>>,
    pub execution_title: Option<String>,
    pub focus: FocusPane,
    pub selected_action: usize,
}

impl UiState {
    pub fn new(is_first_launch: bool, active_theme: ThemeName) -> Self {
        Self {
            phase: if is_first_launch {
                UiPhase::FirstLaunch
            } else {
                UiPhase::Ready
            },
            first_launch_choice: 0,
            scan_progress: 0,
            palette: CommandPaletteState::closed(),
            notifications: Vec::new(),
            active_theme,
            picked_folder: None,
            execution_output: None,
            execution_title: None,
            focus: FocusPane::Sidebar,
            selected_action: 0,
        }
    }

    pub fn selected_first_launch_choice(&self) -> FirstLaunchChoice {
        match self.first_launch_choice {
            0 => FirstLaunchChoice::UseCurrentFolder,
            1 => FirstLaunchChoice::BrowseFolder,
            _ => FirstLaunchChoice::Exit,
        }
    }

    pub fn move_first_launch_next(&mut self) {
        self.first_launch_choice = (self.first_launch_choice + 1) % 3;
    }

    pub fn move_first_launch_previous(&mut self) {
        self.first_launch_choice = if self.first_launch_choice == 0 {
            2
        } else {
            self.first_launch_choice - 1
        };
    }

    pub fn start_scanning(&mut self) {
        self.phase = UiPhase::Scanning;
        self.scan_progress = 0;
    }

    pub fn advance_scan(&mut self) {
        self.scan_progress = (self.scan_progress + 18).min(100);
        if self.scan_progress >= 100 {
            self.phase = UiPhase::Ready;
            self.notifications
                .push(Notification::success("Project scan complete"));
        }
    }

    pub fn push_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
        self.notifications.truncate(3);
    }

    pub fn tick_notifications(&mut self) {
        self.notifications.retain_mut(Notification::tick);
    }

    /// Show execution output from a command in the main area.
    pub fn show_execution_output(&mut self, title: String, lines: Vec<String>) {
        self.execution_title = Some(title);
        self.execution_output = Some(lines);
    }

    /// Clear the execution output panel.
    pub fn clear_execution_output(&mut self) {
        self.execution_title = None;
        self.execution_output = None;
    }

    /// Whether there is execution output to display.
    pub fn has_execution_output(&self) -> bool {
        self.execution_output.is_some()
    }

    /// Toggle focus between sidebar and main panel.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPane::Sidebar => FocusPane::Main,
            FocusPane::Main => FocusPane::Sidebar,
        };
    }

    /// Move action selection down, wrapping around.
    pub fn move_action_next(&mut self, total: usize) {
        if total > 0 {
            self.selected_action = (self.selected_action + 1) % total;
        }
    }

    /// Move action selection up, wrapping around.
    pub fn move_action_previous(&mut self, total: usize) {
        if total > 0 {
            self.selected_action = if self.selected_action == 0 {
                total - 1
            } else {
                self.selected_action - 1
            };
        }
    }

    /// Reset action selection when changing screens.
    pub fn reset_action_selection(&mut self) {
        self.selected_action = 0;
    }

    /// Get the list of actions relevant to the given screen.
    pub fn screen_actions<'a>(
        &self,
        actions: &'a [CommandAction],
        screen: Screen,
    ) -> Vec<&'a CommandAction> {
        actions
            .iter()
            .filter(|a| a.screen == screen)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{ThemeName, UiPhase, UiState};

    #[test]
    fn first_launch_starts_before_ready_dashboard() {
        let state = UiState::new(true, ThemeName::Dark);

        assert_eq!(state.phase, UiPhase::FirstLaunch);
    }

    #[test]
    fn returning_launch_starts_ready_dashboard() {
        let state = UiState::new(false, ThemeName::Dark);

        assert_eq!(state.phase, UiPhase::Ready);
    }

    #[test]
    fn scan_progress_eventually_reaches_ready() {
        let mut state = UiState::new(true, ThemeName::Dark);
        state.start_scanning();

        for _ in 0..6 {
            state.advance_scan();
        }

        assert_eq!(state.phase, UiPhase::Ready);
        assert_eq!(state.scan_progress, 100);
    }

    #[test]
    fn test_execution_output() {
        let mut state = UiState::new(false, ThemeName::Dark);
        assert!(!state.has_execution_output());

        state.show_execution_output("My Title".to_string(), vec!["line 1".to_string()]);
        assert!(state.has_execution_output());
        assert_eq!(state.execution_title, Some("My Title".to_string()));
        assert_eq!(state.execution_output, Some(vec!["line 1".to_string()]));

        state.clear_execution_output();
        assert!(!state.has_execution_output());
        assert!(state.execution_title.is_none());
        assert!(state.execution_output.is_none());
    }
}
