use std::{io, path::PathBuf, time::Duration};

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
        command_palette, folder_picker,
        state::{FirstLaunchChoice, Notification, NotificationLevel, UiPhase},
        statusbar,
        theme::{self, ThemeName},
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

                if state.ui.palette.open {
                    handle_palette_key(state, key.code);
                    continue;
                }

                if state.ui.phase == UiPhase::FirstLaunch {
                    if handle_first_launch_key(state, key.code)? {
                        break;
                    }
                    continue;
                }

                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) => break,
                    (KeyCode::Esc, _) if state.ui.has_execution_output() => {
                        state.ui.clear_execution_output()
                    }
                    (KeyCode::Char('p'), KeyModifiers::CONTROL) => state.ui.palette.open(),
                    (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                        refresh_project(state)?;
                    }
                    (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                        route_action(state, "docker.build")
                    }
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                        route_action(state, "kubernetes.deploy")
                    }
                    (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                        router::route_to(state, Screen::Monitoring)
                    }
                    (KeyCode::Char('t'), _) => cycle_theme(state),
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                        state.ui.clear_execution_output();
                        navigation::move_next(state);
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                        state.ui.clear_execution_output();
                        navigation::move_previous(state);
                    }
                    (KeyCode::Enter, _) => {
                        state.ui.clear_execution_output();
                        router::select_current_menu(state);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn handle_first_launch_key(state: &mut AppState, code: KeyCode) -> io::Result<bool> {
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

fn render(frame: &mut Frame, state: &AppState) {
    let palette = theme::Palette::for_theme(state.ui.active_theme);

    match state.ui.phase {
        UiPhase::FirstLaunch => {
            render_first_launch(frame, frame.area(), state, palette);
            render_notifications(frame, state, palette);
            return;
        }
        UiPhase::Scanning => {
            render_scanning(frame, frame.area(), state, palette);
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

    frame.render_widget(
        List::new(items).block(Block::default().title(" Navigation ").borders(Borders::ALL)),
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
            &format!(" {title} "),
            format!("{content}\n\nPress Esc or navigate to dismiss."),
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

    let actions = state
        .actions
        .iter()
        .take(10)
        .map(|action| {
            let marker = if action.enabled { "-" } else { "x" };
            format!("{marker} {}", action.label)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let analysis = analyzer::ProjectAnalysis::from_context(
        &state.project,
        state.capabilities.clone(),
        state.runtime.clone(),
    );
    let content = format!(
        "Project: {}\nStack: {}\nRoot: {}\n\nRuntime\nDocker: {}\nCluster: {}\n\nAvailable Actions:\n{}\n\nNext Steps:\n{}\n\nCtrl+P opens command palette. Press t to cycle themes.",
        state.project.name,
        state.project.stack,
        state.project.root.display(),
        availability(state.runtime.docker_running),
        availability(state.runtime.cluster_connected),
        if actions.is_empty() { "none".to_string() } else { actions },
        render_short_list(&analysis.next_steps)
    );

    render_panel(
        frame,
        dashboard_layout[1],
        " Project Dashboard ",
        content,
        palette,
    );
}

fn render_docker(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let content = if state.capabilities.docker {
        format!(
            "Dockerfile detected\nDaemon: {}\n\nActions\n- Build Docker Image\n- Run Container\n- Docker Logs\n\n{}",
            availability(state.runtime.docker_running),
            if state.runtime.docker_running {
                "Runtime actions are ready for the Docker engine implementation."
            } else {
                "Start Docker Desktop or the Docker service to enable runtime actions."
            }
        )
    } else {
        empty_state(
            "Docker Not Configured",
            "No Dockerfile was found.",
            "Generate or add a Dockerfile to unlock image and container workflows.",
        )
    };
    render_panel(frame, area, " Docker ", content, palette);
}

fn render_compose(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let content = if state.capabilities.compose {
        "Compose file detected\n\nActions\n- Compose Up\n- Compose Down\n- Compose Logs\n- Restart Services"
            .to_string()
    } else {
        empty_state(
            "Compose Not Configured",
            "No docker-compose.yml or compose.yaml file was found.",
            "Add a Compose file to unlock multi-service workflows.",
        )
    };
    render_panel(frame, area, " Compose ", content, palette);
}

fn render_kubernetes(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let content = if state.capabilities.kubernetes {
        format!(
            "Kubernetes manifests detected\nCluster: {}\n\nResources\n- Deployments\n- Pods\n- Services\n- Ingress\n- ConfigMaps\n- Secrets\n\n{}",
            availability(state.runtime.cluster_connected),
            if state.runtime.cluster_connected {
                "Read-only resource views are ready for kube-rs integration."
            } else {
                "Connect a cluster or start Minikube to enable deployment actions."
            }
        )
    } else {
        empty_state(
            "Kubernetes Not Configured",
            "No deployment, service, ingress, or kustomization file was found.",
            "Generate or add manifests to unlock cluster workflows.",
        )
    };
    render_panel(frame, area, " Kubernetes ", content, palette);
}

fn render_helm(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let content = if state.capabilities.helm {
        "Chart.yaml detected\n\nActions\n- Helm Install\n- Helm Upgrade\n- Helm Rollback"
            .to_string()
    } else {
        empty_state(
            "Helm Not Configured",
            "No Chart.yaml file was found.",
            "Add a chart to unlock Helm workflows.",
        )
    };
    render_panel(frame, area, " Helm ", content, palette);
}

fn render_deployments(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let plan = deploy::pipeline::plan(&state.capabilities, &state.runtime);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
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
        " Deployment Plan ",
        plan.render(),
        palette,
    );
}

fn render_monitoring(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
    let content = if state.capabilities.monitoring {
        format!(
            "Health\nDocker: {}\nCluster: {}\n\nLogs\n- Application Logs\n- Docker Logs\n- Pod Logs\n\nMetrics\n- CPU Usage\n- Memory Usage\n- Network Usage\n\nEvents\nNo events collected yet.",
            availability(state.runtime.docker_running),
            availability(state.runtime.cluster_connected)
        )
    } else {
        empty_state(
            "Monitoring Not Available",
            "No Docker, Compose, or Kubernetes assets were found.",
            "Add runtime configuration to unlock logs, health, metrics, and events.",
        )
    };
    render_panel(frame, area, " Monitoring ", content, palette);
}

fn render_settings(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
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
    render_panel(frame, area, " Settings ", content, palette);
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

fn render_first_launch(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
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

fn render_scanning(frame: &mut Frame, area: Rect, state: &AppState, palette: theme::Palette) {
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

fn found(value: bool) -> &'static str {
    if value {
        "found"
    } else {
        "missing"
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
        let res = handle_first_launch_key(&mut state, KeyCode::Down);
        assert!(res.is_ok());
        assert_eq!(state.ui.first_launch_choice, 1);

        let res = handle_first_launch_key(&mut state, KeyCode::Up);
        assert!(res.is_ok());
        assert_eq!(state.ui.first_launch_choice, 0);

        let res = handle_first_launch_key(&mut state, KeyCode::Enter);
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
