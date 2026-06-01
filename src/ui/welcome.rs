use std::{io, path::PathBuf};

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{startup, state::AppState},
    ui::{
        folder_picker,
        state::{FirstLaunchChoice, Notification},
        theme,
    },
};

pub fn handle_first_launch_key(state: &mut AppState, code: KeyCode) -> io::Result<bool> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => Ok(true),
        KeyCode::Down | KeyCode::Char('j') => {
            state.ui.move_first_launch_next();
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.ui.move_first_launch_previous();
            Ok(false)
        }
        KeyCode::Enter => {
            match state.ui.selected_first_launch_choice() {
                FirstLaunchChoice::UseCurrentFolder => {
                    state.ui.start_scanning();
                    state.notify_info("Scanning current folder");
                }
                FirstLaunchChoice::BrowseFolder => {
                    if let Some(path) = folder_picker::pick_folder()? {
                        reload_project(state, path)?;
                    } else {
                        state.notify_warning("Folder selection cancelled");
                    }
                }
                FirstLaunchChoice::Exit => return Ok(true),
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn reload_project(state: &mut AppState, path: PathBuf) -> io::Result<()> {
    let mut new_state = startup::initialize(path.clone()).map_err(io::Error::other)?;
    new_state.ui.active_theme = state.ui.active_theme;
    new_state.ui.picked_folder = Some(path.clone());
    new_state.ui.start_scanning();
    new_state
        .ui
        .push_notification(Notification::info(format!("Selected {}", path.display())));
    *state = new_state;
    Ok(())
}

pub fn render_first_launch(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let welcome_area = welcome_rect(area);
    let outer_block = render_outer_block(frame, welcome_area, palette);
    let inner_area = outer_block.inner(welcome_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // ASCII art
            Constraint::Length(4), // Subtitle, link, author
            Constraint::Length(8), // Project card
            Constraint::Min(5),    // Options
        ])
        .split(inner_area);

    render_ascii_banner(frame, chunks[0], palette);
    render_subtitle(frame, chunks[1], palette);
    render_capabilities_card(frame, chunks[2], state, palette);

    // 4. Action/Choice List
    let choices = [
        FirstLaunchChoice::UseCurrentFolder,
        FirstLaunchChoice::BrowseFolder,
        FirstLaunchChoice::Exit,
    ];
    let items = choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let marker = if state.ui.first_launch_choice == index {
                "> "
            } else {
                "  "
            };
            let style = if state.ui.first_launch_choice == index {
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.text)
            };
            ListItem::new(format!("{marker}{}", choice.label())).style(style)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(Block::default().title(" Actions ").borders(Borders::ALL)),
        chunks[3],
    );
}

pub fn render_scanning(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let area = centered_rect(62, 36, area);
    let content = format!(
        "Scanning Project...\n\nProject: {}\nRoot: {}\n\nDockerfile: {}\nCompose: {}\nKubernetes: {}\nHelm: {}\nStack: {}",
        state.project.name,
        state.project.root.display(),
        found(state.capabilities.docker),
        found(state.capabilities.compose),
        found(state.capabilities.kubernetes),
        found(state.capabilities.helm),
        state.project.stack
    );
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(3)])
        .split(area);

    frame.render_widget(Clear, area);
    render_panel(frame, layout[0], " Project Scan ", content, palette);
    frame.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(palette.accent))
            .percent(state.ui.scan_progress),
        layout[1],
    );
}

fn welcome_rect(area: Rect) -> Rect {
    let width_u32 = (area.width as u32 * 65 / 100)
        .max(60)
        .min(area.width as u32);
    let height_u32 = 25u32.min(area.height as u32).max(20);

    let width = width_u32 as u16;
    let height = height_u32 as u16;
    let x = area.width.saturating_sub(width) / 2;
    let y = area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn render_outer_block(
    frame: &mut Frame,
    welcome_area: Rect,
    palette: theme::Palette,
) -> Block<'static> {
    let outer_block = Block::default().borders(Borders::ALL).title(Span::styled(
        " KDC - Welcome ",
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Clear, welcome_area);
    frame.render_widget(outer_block.clone(), welcome_area);
    outer_block
}

fn render_ascii_banner(frame: &mut Frame, chunk: Rect, palette: theme::Palette) {
    let ascii_art = vec![
        Line::from(Span::styled(
            "  _  ______   ____ ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " | |/ /  _ \\ / ___|",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " | ' /| | | | |    ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " | . \\| |_| | |___ ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " |_|\\_\\____/ \\____|",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    frame.render_widget(
        Paragraph::new(ascii_art).alignment(Alignment::Center),
        chunk,
    );
}

fn render_subtitle(frame: &mut Frame, chunk: Rect, palette: theme::Palette) {
    let subtitle_info = vec![
        Line::from(Span::styled(
            "Kubernetes & Docker Commander like a boss.",
            Style::default().fg(palette.text),
        )),
        Line::from(Span::styled(
            "https://github.com/KDM-cli/kdc-cli",
            Style::default().fg(palette.muted),
        )),
        Line::from(vec![
            Span::raw("[with "),
            Span::styled("♥", Style::default().fg(palette.danger)),
            Span::raw(" by "),
            Span::styled("@utkarsh232005", Style::default().fg(palette.success)),
            Span::raw("]"),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(subtitle_info).alignment(Alignment::Center),
        chunk,
    );
}

fn capability_line(label: &str, present: bool, palette: theme::Palette) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {}: ", label), Style::default().fg(palette.muted)),
        Span::styled(
            if present { "Found" } else { "Missing" },
            Style::default().fg(if present {
                palette.success
            } else {
                palette.warning
            }),
        ),
    ])
}

fn render_capabilities_card(
    frame: &mut Frame,
    chunk: Rect,
    state: &AppState,
    palette: theme::Palette,
) {
    let mut details = Vec::new();
    details.push(Line::from(vec![
        Span::styled("  Root: ", Style::default().fg(palette.muted)),
        Span::styled(
            format!("{}", state.project.root.display()),
            Style::default().fg(palette.text),
        ),
    ]));
    details.push(Line::from(vec![
        Span::styled("  Stack: ", Style::default().fg(palette.muted)),
        Span::styled(
            format!("{}", state.project.stack),
            Style::default().fg(palette.text),
        ),
    ]));
    details.push(capability_line(
        "Dockerfile",
        state.capabilities.docker,
        palette,
    ));
    details.push(capability_line(
        "Compose",
        state.capabilities.compose,
        palette,
    ));
    details.push(capability_line(
        "Kubernetes",
        state.capabilities.kubernetes,
        palette,
    ));
    details.push(capability_line(
        "Helm Chart",
        state.capabilities.helm,
        palette,
    ));

    frame.render_widget(
        Paragraph::new(details)
            .block(
                Block::default()
                    .title(" Current Directory Details ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(palette.text)),
        chunk,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn found(value: bool) -> &'static str {
    if value {
        "found"
    } else {
        "missing"
    }
}

fn render_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    content: String,
    palette: theme::Palette,
) {
    frame.render_widget(
        Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(palette.text).bg(palette.background)),
        area,
    );
}
