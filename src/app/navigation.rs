use crate::app::state::AppState;

pub fn move_next(state: &mut AppState) {
    if state.menus.is_empty() {
        state.selected_menu = 0;
        return;
    }

    state.selected_menu = (state.selected_menu + 1) % state.menus.len();
}

pub fn move_previous(state: &mut AppState) {
    if state.menus.is_empty() {
        state.selected_menu = 0;
        return;
    }

    state.selected_menu = if state.selected_menu == 0 {
        state.menus.len() - 1
    } else {
        state.selected_menu - 1
    };
}
