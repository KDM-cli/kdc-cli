use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

use crate::{
    app::{dashboard, navigation, router, state::AppState},
    ui::{statusbar, theme},
};

pub fn run(mut state: AppState) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &mut state);

    ratatui::restore();
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

fn run_loop(terminal: &mut DefaultTerminal, state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, state))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) => break,
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => navigation::move_next(state),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => navigation::move_previous(state),
                    (KeyCode::Enter, _) => router::select_current_menu(state),
                    (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                        state.status_message = "Refresh requested".to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, state: &AppState) {
    let palette = theme::Palette::default();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_header(frame, root[0], state, palette);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(30)])
        .split(root[1]);

    render_sidebar(frame, body[0], state, palette);
    render_main(frame, body[1], state, palette);
    statusbar::render(frame, root[2], state, palette);
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let title = Line::from(vec![
        Span::styled(
            "KDC",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Kubernetes Docker Commander  "),
        Span::styled(
            state.current_screen.title(),
            Style::default().fg(palette.success),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM)),
        area,
    );
}

fn render_sidebar(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let items = state
        .menus
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let marker = if index == state.selected_menu {
                "> "
            } else {
                "  "
            };
            let style = if !item.enabled {
                Style::default().fg(palette.muted)
            } else if index == state.selected_menu {
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(format!("{marker}{}", item.label))).style(style)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(Block::default().title(" Navigation ").borders(Borders::ALL)),
        area,
    );
}

fn render_main(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let cards = dashboard::cards(state)
        .into_iter()
        .map(|card| format!("{}: {}", card.title, card.value))
        .collect::<Vec<_>>()
        .join("\n");

    let capabilities = state.capabilities.summary();
    let actions = state
        .actions
        .iter()
        .take(8)
        .map(|action| {
            let marker = if action.enabled { "-" } else { "x" };
            format!("{marker} {}", action.label)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!(
        "{cards}\n\n{}\n\n{}\n\nAvailable Actions:\n{}\n\nTheme: {}\nRecent Projects: {}\n\nUse Up/Down to navigate, Enter to open, q to quit.",
        state.project.summary(),
        capabilities,
        if actions.is_empty() { "none".to_string() } else { actions },
        state.settings.theme,
        state.history.recent_projects.len()
    );

    frame.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Project Dashboard ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(palette.text)),
        area,
    );
}
