use crate::domain::screen::Screen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub visible: bool,
    pub enabled: bool,
    pub screen: Screen,
    pub shortcut: Option<String>,
}

impl MenuItem {
    pub fn visible(id: &str, label: &str, screen: Screen, shortcut: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            visible: true,
            enabled: true,
            screen,
            shortcut: shortcut.map(str::to_string),
        }
    }

    pub fn disabled(id: &str, label: &str, screen: Screen) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            visible: true,
            enabled: false,
            screen,
            shortcut: None,
        }
    }

    pub fn display_line(&self) -> String {
        let state = if self.enabled { "enabled" } else { "disabled" };
        match &self.shortcut {
            Some(shortcut) => format!("{} [{}] - {}", self.label, shortcut, state),
            None => format!("{} - {}", self.label, state),
        }
    }
}
