use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app::state::AppState, ui::theme::Palette};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, palette: Palette) {
    let cluster = if state.runtime.cluster_connected {
        "cluster connected"
    } else {
        "cluster offline"
    };
    let docker = if state.runtime.docker_running {
        "docker running"
    } else {
        "docker unknown"
    };
    let text = format!(
        "Project: {} | Stack: {} | {} | {} | {}",
        state.project.name, state.project.stack, docker, cluster, state.status_message
    );

    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(palette.muted))
            .block(Block::default().borders(Borders::TOP)),
        area,
    );
}
