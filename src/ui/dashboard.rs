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
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

use crate::{
    app::{dashboard, navigation, router, startup, state::AppState},
    deploy,
    domain::screen::Screen,
    project::analyzer,
    services::executor::{CommandExecutor, KdcExecutor},
    ui::{
        command_palette,
        state::{FocusPane, Notification, NotificationLevel, UiPhase},
        statusbar,
        theme::{self, ThemeName},
        welcome,
    },
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
        if state.ui.phase == UiPhase::Scanning {
            state.ui.advance_scan();
        }
        state.ui.tick_notifications();

        terminal.draw(|frame| render(frame, state))?;

        if event::poll(Duration::from_millis(180))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if handle_key_event(state, key)? {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn handle_key_event(state: &mut AppState, key: event::KeyEvent) -> io::Result<bool> {
    if state.ui.palette.open {
        handle_palette_key(state, key.code);
        return Ok(false);
    }

    if state.ui.phase == UiPhase::FirstLaunch {
        return welcome::handle_first_launch_key(state, key.code);
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => return Ok(true),
        (KeyCode::Esc, _) if state.ui.has_execution_output() => state.ui.clear_execution_output(),
        (KeyCode::Char('p'), KeyModifiers::CONTROL) => state.ui.palette.open(),
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            refresh_project(state)?;
        }
        (KeyCode::Char('b'), KeyModifiers::CONTROL) => route_action(state, "docker.build"),
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => route_action(state, "kubernetes.deploy"),
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => router::route_to(state, Screen::Monitoring),
        (KeyCode::Char('t'), _) => cycle_theme(state),
        (KeyCode::Tab, _) | (KeyCode::BackTab, _) => {
            state.ui.toggle_focus();
        }
        (KeyCode::Left, _) => {
            state.ui.focus = FocusPane::Sidebar;
        }
        (KeyCode::Right, _) => {
            state.ui.focus = FocusPane::Main;
        }
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
            handle_vertical_key(state, VerticalDirection::Next);
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
            handle_vertical_key(state, VerticalDirection::Previous);
        }
        (KeyCode::Enter, _) => {
            handle_enter_key(state);
        }
        _ => {}
    }
    Ok(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerticalDirection {
    Next,
    Previous,
}

fn handle_vertical_key(state: &mut AppState, direction: VerticalDirection) {
    state.ui.clear_execution_output();
    match state.ui.focus {
        FocusPane::Sidebar => match direction {
            VerticalDirection::Next => navigation::move_next(state),
            VerticalDirection::Previous => navigation::move_previous(state),
        },
        FocusPane::Main => {
            let total = state
                .ui
                .screen_actions(&state.actions, state.current_screen)
                .len();
            match direction {
                VerticalDirection::Next => state.ui.move_action_next(total),
                VerticalDirection::Previous => state.ui.move_action_previous(total),
            }
        }
    }
}

fn handle_enter_key(state: &mut AppState) {
    state.ui.clear_execution_output();
    match state.ui.focus {
        FocusPane::Sidebar => {
            state.ui.reset_action_selection();
            router::select_current_menu(state);
        }
        FocusPane::Main => {
            let screen_actions: Vec<_> = state
                .ui
                .screen_actions(&state.actions, state.current_screen)
                .iter()
                .map(|a| {
                    (
                        a.id.clone(),
                        a.screen,
                        a.label.clone(),
                        a.enabled,
                        a.reason.clone(),
                    )
                })
                .collect();
            if let Some((id, screen, label, enabled, reason)) =
                screen_actions.get(state.ui.selected_action).cloned()
            {
                if enabled {
                    execute_action(state, &id, screen, &label);
                } else {
                    state.ui.push_notification(Notification::warning(
                        reason.unwrap_or_else(|| "Action unavailable".to_string()),
                    ));
                }
            }
        }
    }
}

fn render(frame: &mut Frame, state: &AppState) {
    let palette = theme::Palette::for_theme(state.ui.active_theme);

    match state.ui.phase {
        UiPhase::FirstLaunch => {
            welcome::render_first_launch(frame, frame.area(), state, palette);
            render_notifications(frame, state, palette);
            return;
        }
        UiPhase::Scanning => {
            welcome::render_scanning(frame, frame.area(), state, palette);
            render_notifications(frame, state, palette);
            return;
        }
        UiPhase::Ready => {}
    }

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
    render_notifications(frame, state, palette);

    if state.ui.palette.open {
        render_command_palette(frame, centered_rect(72, 55, frame.area()), state, palette);
    }
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
        Span::raw("  "),
        Span::styled(
            state.ui.active_theme.label(),
            Style::default().fg(palette.muted),
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
    let is_focused = state.ui.focus == FocusPane::Sidebar;
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
                Style::default().fg(palette.text)
            };

            ListItem::new(Line::from(format!("{marker}{}", item.label))).style(style)
        })
        .collect::<Vec<_>>();

    let border_style = if is_focused {
        Style::default().fg(palette.accent)
    } else {
        Style::default().fg(palette.muted)
    };
    let title = if is_focused {
        " Navigation ● "
    } else {
        " Navigation "
    };

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        ),
        area,
    );
}

fn render_main(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    // If there is execution output, show it in the main area.
    if let Some(lines) = &state.ui.execution_output {
        let title = state
            .ui
            .execution_title
            .as_deref()
            .unwrap_or("Execution Output");
        let content = lines.join("\n");
        render_panel(
            frame,
            area,
            PanelContent {
                title: format!(" {title} "),
                body: format!("{content}\n\nPress Esc or navigate to dismiss."),
            },
            palette,
        );
        return;
    }

    match state.current_screen {
        Screen::Dashboard => render_dashboard(frame, area, state, palette),
        Screen::Docker => render_docker(frame, area, state, palette),
        Screen::Compose => render_compose(frame, area, state, palette),
        Screen::Kubernetes => render_kubernetes(frame, area, state, palette),
        Screen::Helm => render_helm(frame, area, state, palette),
        Screen::Deployments => render_deployments(frame, area, state, palette),
        Screen::Monitoring => render_monitoring(frame, area, state, palette),
        Screen::Settings => render_settings(frame, area, state, palette),
    }
}

fn render_dashboard(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let dashboard_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(8)])
        .split(area);
    let card_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(dashboard_layout[0]);
    let cards = dashboard::cards(state)
        .into_iter()
        .take(4)
        .collect::<Vec<_>>();

    for (index, area) in card_area.iter().enumerate() {
        let content = cards
            .get(index)
            .map(|card| format!("{}\n{}", card.title, card.value))
            .unwrap_or_else(|| "Status\nPending".to_string());
        frame.render_widget(
            Paragraph::new(content)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(palette.text).bg(palette.background)),
            *area,
        );
    }

    let analysis = analyzer::ProjectAnalysis::from_context(
        &state.project,
        state.capabilities.clone(),
        state.runtime.clone(),
    );

    // Split the bottom area into info panel and action list
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(dashboard_layout[1]);

    // Left: project info
    let info_content = format!(
        "Project: {}\nStack: {}\nRoot: {}\n\nRuntime\nDocker: {}\nCluster: {}\n\nNext Steps:\n{}\n\nCtrl+P opens command palette.\nPress t to cycle themes.\nTab/←/→ to switch panes.",
        state.project.name,
        state.project.stack,
        state.project.root.display(),
        availability(state.runtime.docker_running),
        availability(state.runtime.cluster_connected),
        render_short_list(&analysis.next_steps)
    );

    render_panel(
        frame,
        bottom_layout[0],
        PanelContent {
            title: " Project Info ".to_string(),
            body: info_content,
        },
        palette,
    );

    // Right: selectable action list
    render_action_list(frame, bottom_layout[1], state, palette, Screen::Dashboard);
}

fn render_capability_screen(
    frame: &mut Frame,
    state: &AppState,
    palette: theme::Palette,
    spec: CapabilityScreenSpec<'_>,
) {
    if spec.has_capability {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(spec.info_height), Constraint::Min(5)])
            .split(spec.area);

        render_panel(
            frame,
            layout[0],
            PanelContent {
                title: spec.info_title.to_string(),
                body: spec.info_content,
            },
            palette,
        );
        render_action_list(frame, layout[1], state, palette, spec.screen);
    } else {
        let content = empty_state(spec.empty_title, spec.empty_body, spec.empty_suggestion);
        render_panel(
            frame,
            spec.area,
            PanelContent {
                title: spec.panel_title.to_string(),
                body: content,
            },
            palette,
        );
    }
}

struct CapabilityScreenSpec<'a> {
    area: Rect,
    has_capability: bool,
    screen: Screen,
    info_title: &'a str,
    info_content: String,
    panel_title: &'a str,
    empty_title: &'a str,
    empty_body: &'a str,
    empty_suggestion: &'a str,
    info_height: u16,
}

fn render_docker(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let info = format!(
        "Dockerfile detected\nDaemon: {}\n\n{}",
        availability(state.runtime.docker_running),
        if state.runtime.docker_running {
            "Runtime actions are ready for the Docker engine implementation."
        } else {
            "Start Docker Desktop or the Docker service to enable runtime actions."
        }
    );
    let spec = CapabilityScreenSpec {
        area,
        has_capability: state.capabilities.docker,
        screen: Screen::Docker,
        info_title: " Docker Info ",
        info_content: info,
        panel_title: " Docker ",
        empty_title: "Docker Not Configured",
        empty_body: "No Dockerfile was found.",
        empty_suggestion: "Generate or add a Dockerfile to unlock image and container workflows.",
        info_height: 8,
    };
    render_capability_screen(frame, state, palette, spec);
}

fn render_compose(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let spec = CapabilityScreenSpec {
        area,
        has_capability: state.capabilities.compose,
        screen: Screen::Compose,
        info_title: " Compose Info ",
        info_content: "Compose file detected".to_string(),
        panel_title: " Compose ",
        empty_title: "Compose Not Configured",
        empty_body: "No docker-compose.yml or compose.yaml file was found.",
        empty_suggestion: "Add a Compose file to unlock multi-service workflows.",
        info_height: 5,
    };
    render_capability_screen(frame, state, palette, spec);
}

fn render_kubernetes(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let info = format!(
        "Kubernetes manifests detected\nCluster: {}\n\nResources\n- Deployments\n- Pods\n- Services\n\n{}",
        availability(state.runtime.cluster_connected),
        if state.runtime.cluster_connected {
            "Read-only resource views are ready for kube-rs integration."
        } else {
            "Connect a cluster or start Minikube to enable deployment actions."
        }
    );
    let spec = CapabilityScreenSpec {
        area,
        has_capability: state.capabilities.kubernetes,
        screen: Screen::Kubernetes,
        info_title: " Kubernetes Info ",
        info_content: info,
        panel_title: " Kubernetes ",
        empty_title: "Kubernetes Not Configured",
        empty_body: "No deployment, service, ingress, or kustomization file was found.",
        empty_suggestion: "Generate or add manifests to unlock cluster workflows.",
        info_height: 10,
    };
    render_capability_screen(frame, state, palette, spec);
}

fn render_helm(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let spec = CapabilityScreenSpec {
        area,
        has_capability: state.capabilities.helm,
        screen: Screen::Helm,
        info_title: " Helm Info ",
        info_content: "Chart.yaml detected".to_string(),
        panel_title: " Helm ",
        empty_title: "Helm Not Configured",
        empty_body: "No Chart.yaml file was found.",
        empty_suggestion: "Add a chart to unlock Helm workflows.",
        info_height: 5,
    };
    render_capability_screen(frame, state, palette, spec);
}

fn render_deployments(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let plan = deploy::pipeline::plan(&state.capabilities, &state.runtime);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(5),
            Constraint::Length(8),
        ])
        .split(area);
    let ready = plan.ready();

    frame.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(" Deployment Progress ")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(if ready {
                palette.success
            } else {
                palette.warning
            }))
            .ratio(if ready { 1.0 } else { 0.45 })
            .label(if ready { "Ready" } else { "Blocked" }),
        layout[0],
    );
    render_panel(
        frame,
        layout[1],
        PanelContent {
            title: " Deployment Plan ".to_string(),
            body: plan.render(),
        },
        palette,
    );
    render_action_list(frame, layout[2], state, palette, Screen::Deployments);
}

fn render_monitoring(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let info = format!(
        "Health\nDocker: {}\nCluster: {}\n\nMetrics\n- CPU Usage\n- Memory Usage\n- Network Usage\n\nEvents\nNo events collected yet.",
        availability(state.runtime.docker_running),
        availability(state.runtime.cluster_connected)
    );
    let spec = CapabilityScreenSpec {
        area,
        has_capability: state.capabilities.monitoring,
        screen: Screen::Monitoring,
        info_title: " Monitoring Info ",
        info_content: info,
        panel_title: " Monitoring ",
        empty_title: "Monitoring Not Available",
        empty_body: "No Docker, Compose, or Kubernetes assets were found.",
        empty_suggestion: "Add runtime configuration to unlock logs, health, metrics, and events.",
        info_height: 10,
    };
    render_capability_screen(frame, state, palette, spec);
}

fn render_settings(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let settings_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(5)])
        .split(area);

    let content = format!(
        "Project: {}\nTheme: {}\nDefault Environment: {}\nRecent Projects: {}\n\nTheme Options\n{}\n\nPress t to cycle theme.",
        state.project.name,
        state.ui.active_theme.label(),
        state.settings.default_environment,
        state.history.recent_projects.len(),
        ThemeName::ALL
            .iter()
            .map(|theme| format!("- {}", theme.label()))
            .collect::<Vec<_>>()
            .join("\n")
    );
    render_panel(
        frame,
        settings_layout[0],
        PanelContent {
            title: " Settings ".to_string(),
            body: content,
        },
        palette,
    );
    render_action_list(frame, settings_layout[1], state, palette, Screen::Settings);
}

struct PanelContent {
    title: String,
    body: String,
}

fn render_panel(frame: &mut Frame, area: Rect, content: PanelContent, palette: theme::Palette) {
    frame.render_widget(
        Paragraph::new(content.body)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(content.title).borders(Borders::ALL))
            .style(Style::default().fg(palette.text).bg(palette.background)),
        area,
    );
}

fn render_action_list(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    palette: theme::Palette,
    screen: Screen,
) {
    let is_focused = state.ui.focus == FocusPane::Main;
    let screen_actions = state.ui.screen_actions(&state.actions, screen);

    if screen_actions.is_empty() {
        render_empty_actions(frame, area, is_focused, palette);
    } else {
        let items = build_action_list_items(
            &screen_actions,
            is_focused,
            state.ui.selected_action,
            palette,
        );
        render_active_actions(frame, area, items, is_focused, palette);
    }
}

fn render_empty_actions(frame: &mut Frame, area: Rect, is_focused: bool, palette: theme::Palette) {
    let border_style = if is_focused {
        Style::default().fg(palette.accent)
    } else {
        Style::default().fg(palette.muted)
    };
    frame.render_widget(
        Paragraph::new(
            "No actions available for this screen.\n\nUse Ctrl+P to open the command palette.",
        )
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(" Actions ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(palette.muted)),
        area,
    );
}

fn build_action_list_items(
    actions: &[&crate::commands::palette::CommandAction],
    is_focused: bool,
    selected_idx: usize,
    palette: theme::Palette,
) -> Vec<ListItem<'static>> {
    actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let is_selected = is_focused && index == selected_idx;
            let marker = if is_selected { "▸ " } else { "  " };

            let style = if !action.enabled {
                Style::default().fg(palette.muted)
            } else if is_selected {
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.text)
            };

            let suffix = if !action.enabled {
                " (unavailable)"
            } else {
                ""
            };

            ListItem::new(Line::from(format!("{marker}{}{suffix}", action.label))).style(style)
        })
        .collect()
}

fn render_active_actions(
    frame: &mut Frame,
    area: Rect,
    items: Vec<ListItem<'static>>,
    is_focused: bool,
    palette: theme::Palette,
) {
    let border_style = if is_focused {
        Style::default().fg(palette.accent)
    } else {
        Style::default().fg(palette.muted)
    };
    let title = if is_focused {
        " Actions ● (Enter to run) "
    } else {
        " Actions (Tab to focus) "
    };

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        ),
        area,
    );
}

fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    palette: theme::Palette,
) {
    let matches = command_palette::search_actions(&state.actions, &state.ui.palette.query);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);
    let query = format!("> {}", state.ui.palette.query);
    let items = matches
        .iter()
        .take(8)
        .enumerate()
        .map(|(index, action)| {
            let style = if index == state.ui.palette.selected {
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.text)
            };
            ListItem::new(action.label.clone()).style(style)
        })
        .collect::<Vec<_>>();

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(query)
            .block(
                Block::default()
                    .title(" Run Command ")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(palette.text)),
        layout[0],
    );
    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL)),
        layout[1],
    );
}

fn render_notifications(frame: &mut Frame, state: &AppState, palette: theme::Palette) {
    if state.ui.notifications.is_empty() {
        return;
    }

    let height = state.ui.notifications.len() as u16 + 2;
    let area = Rect {
        x: frame.area().width.saturating_sub(42),
        y: frame.area().height.saturating_sub(height + 2),
        width: 40,
        height,
    };
    let content = state
        .ui
        .notifications
        .iter()
        .map(|notification| notification.message.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let color = state
        .ui
        .notifications
        .last()
        .map(|notification| match notification.level {
            NotificationLevel::Info => palette.accent,
            NotificationLevel::Success => palette.success,
            NotificationLevel::Warning => palette.warning,
        })
        .unwrap_or(palette.accent);

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(content)
            .block(Block::default().title(" Notice ").borders(Borders::ALL))
            .style(Style::default().fg(color)),
        area,
    );
}

fn handle_palette_key(state: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Esc => state.ui.palette.close(),
        KeyCode::Backspace => {
            state.ui.palette.query.pop();
            state.ui.palette.selected = 0;
        }
        KeyCode::Char(ch) => {
            state.ui.palette.query.push(ch);
            state.ui.palette.selected = 0;
        }
        KeyCode::Down => {
            let total = command_palette::search_actions(&state.actions, &state.ui.palette.query)
                .len()
                .min(8);
            if total > 0 {
                state.ui.palette.selected = (state.ui.palette.selected + 1) % total;
            }
        }
        KeyCode::Up => {
            let total = command_palette::search_actions(&state.actions, &state.ui.palette.query)
                .len()
                .min(8);
            if total > 0 {
                state.ui.palette.selected = if state.ui.palette.selected == 0 {
                    total - 1
                } else {
                    state.ui.palette.selected - 1
                };
            }
        }
        KeyCode::Enter => {
            let selected = command_palette::search_actions(&state.actions, &state.ui.palette.query)
                .get(state.ui.palette.selected)
                .map(|action| (action.id.clone(), action.screen, action.label.clone()));
            if let Some((id, screen, label)) = selected {
                state.ui.palette.close();
                execute_action(state, &id, screen, &label);
            }
        }
        _ => {}
    }
}

fn route_action(state: &mut AppState, id: &str) {
    let action = state
        .actions
        .iter()
        .find(|action| action.id == id)
        .map(|action| {
            (
                action.screen,
                action.enabled,
                action.label.clone(),
                action.reason.clone(),
            )
        });
    if let Some((screen, enabled, label, reason)) = action {
        if enabled {
            execute_action(state, id, screen, &label);
        } else {
            state.ui.push_notification(Notification::warning(
                reason.unwrap_or_else(|| "Action unavailable".to_string()),
            ));
        }
    }
}

fn execute_action(state: &mut AppState, id: &str, screen: Screen, label: &str) {
    router::route_to(state, screen);
    let executor = KdcExecutor::new(&state.project);
    match executor.execute(id) {
        Ok(result) => {
            state.status_message = result.message.clone();
            let mut output = if result.output_lines.is_empty() {
                vec![result.message.clone()]
            } else {
                result.output_lines
            };
            if !result.success {
                output.insert(0, result.message.clone());
            }
            state.ui.show_execution_output(label.to_string(), output);
            if result.success {
                state
                    .ui
                    .push_notification(Notification::success(result.message));
            } else {
                state
                    .ui
                    .push_notification(Notification::warning(result.message));
            }
        }
        Err(err) => {
            let message = format!("{label} failed: {err}");
            state.status_message = message.clone();
            state
                .ui
                .show_execution_output(label.to_string(), vec![message.clone()]);
            state.ui.push_notification(Notification::warning(message));
        }
    }
}

fn refresh_project(state: &mut AppState) -> io::Result<()> {
    let root = state.project.root.clone();
    let active_theme = state.ui.active_theme;
    let mut refreshed = startup::initialize(root).map_err(io::Error::other)?;
    refreshed.ui.active_theme = active_theme;
    refreshed.ui.start_scanning();
    refreshed
        .ui
        .push_notification(Notification::info("Project and runtime refreshed"));
    *state = refreshed;
    Ok(())
}

fn cycle_theme(state: &mut AppState) {
    state.ui.active_theme = state.ui.active_theme.next();
    let theme_str = state
        .ui
        .active_theme
        .label()
        .to_lowercase()
        .replace(' ', "-");
    state.settings.theme = theme_str;

    // Persist the theme change to the config file.
    let config_path = crate::config::paths::config_file();
    if let Err(err) = state.settings.save(&config_path) {
        state.ui.push_notification(Notification::warning(format!(
            "Could not save theme: {err}"
        )));
    }

    state.ui.push_notification(Notification::info(format!(
        "Theme: {}",
        state.ui.active_theme.label()
    )));
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

fn availability(value: bool) -> &'static str {
    if value {
        "available"
    } else {
        "unavailable"
    }
}

fn render_short_list(values: &[String]) -> String {
    if values.is_empty() {
        "- No immediate next steps".to_string()
    } else {
        values
            .iter()
            .take(4)
            .map(|value| format!("- {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn empty_state(title: &str, body: &str, suggestion: &str) -> String {
    format!("{title}\n\n{body}\n\nSuggestion:\n{suggestion}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_render_all_phases() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::utils::test_support::set_mock_path();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = crate::app::startup::initialize(std::path::PathBuf::from(".")).unwrap();

        // 1. First Launch
        state.ui.phase = UiPhase::FirstLaunch;
        let res = terminal.draw(|frame| {
            render(frame, &state);
        });
        assert!(res.is_ok());

        // 2. Scanning
        state.ui.phase = UiPhase::Scanning;
        let res = terminal.draw(|frame| {
            render(frame, &state);
        });
        assert!(res.is_ok());

        // 3. Ready (main screen)
        state.ui.phase = UiPhase::Ready;
        let res = terminal.draw(|frame| {
            render(frame, &state);
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_first_launch_key() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::utils::test_support::set_mock_path();
        let mut state = crate::app::startup::initialize(std::path::PathBuf::from(".")).unwrap();
        state.ui.first_launch_choice = 0;
        let res = welcome::handle_first_launch_key(&mut state, KeyCode::Down);
        assert!(res.is_ok());
        assert_eq!(state.ui.first_launch_choice, 1);

        let res = welcome::handle_first_launch_key(&mut state, KeyCode::Up);
        assert!(res.is_ok());
        assert_eq!(state.ui.first_launch_choice, 0);

        let res = welcome::handle_first_launch_key(&mut state, KeyCode::Enter);
        assert!(res.is_ok());
    }

    #[test]
    fn test_cycle_theme() {
        let _guard = TEST_MUTEX.lock().unwrap();
        crate::utils::test_support::set_mock_path();
        let mut state = crate::app::startup::initialize(std::path::PathBuf::from(".")).unwrap();
        let initial_theme = state.ui.active_theme;
        cycle_theme(&mut state);
        assert_ne!(state.ui.active_theme, initial_theme);
    }
}
