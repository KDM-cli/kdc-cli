use crate::{app::state::AppState, domain::screen::Screen};

pub fn select_current_menu(state: &mut AppState) {
    if let Some(item) = state.selected_menu() {
        let screen = item.screen;
        let label = item.label.clone();

        state.current_screen = screen;
        state.status_message = format!("Opened {label}");
    }
}

pub fn route_to(state: &mut AppState, screen: Screen) {
    state.current_screen = screen;
}
