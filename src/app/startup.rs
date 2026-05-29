use std::path::PathBuf;

use anyhow::Result;

use crate::{
    app::state::{AppState, AppStateInit},
    commands::palette,
    config::{paths, settings::Settings},
    project::{capabilities, detector, runtime},
    storage::history::ProjectHistory,
    ui::menus::CapabilityMenuGenerator,
};

pub fn initialize(project_path: PathBuf) -> Result<AppState> {
    initialize_with_options(
        project_path,
        StartupOptions {
            force_first_launch: false,
        },
    )
}

pub fn initialize_with_options(project_path: PathBuf, options: StartupOptions) -> Result<AppState> {
    let project = detector::detect(project_path)?;
    let capabilities = capabilities::from_project(&project);
    let runtime = runtime::detect(&capabilities);
    let menus = CapabilityMenuGenerator::generate(&capabilities, &runtime);
    let actions = palette::generate_actions(&project, &capabilities, &runtime);
    let settings = Settings::load_or_default(&paths::config_file())?;
    let mut history = ProjectHistory::load_or_default(&paths::history_file())?;
    let is_first_launch = options.force_first_launch || history.recent_projects.is_empty();
    history.record_project(project.root.clone());
    history.save(&paths::history_file())?;

    Ok(AppState::new(AppStateInit {
        project,
        capabilities,
        runtime,
        menus,
        actions,
        settings,
        history,
        is_first_launch,
    }))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StartupOptions {
    pub force_first_launch: bool,
}
