use std::path::PathBuf;

use clap::{Parser, Subcommand};
use kdc::{app::startup, config, deploy, doctor, project::analyzer, ui};

#[derive(Debug, Parser)]
#[command(name = "kdc")]
#[command(about = "Kubernetes Docker Commander")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short, long, value_name = "PATH", default_value = ".")]
    project: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print detected project capabilities.
    Scan,
    /// Print a project analysis report with recommended next steps.
    Analysis,
    /// Print the generated dynamic menus.
    Menus,
    /// Print generated actions, optionally filtered by a search query.
    Actions {
        #[arg(value_name = "QUERY")]
        query: Option<String>,
    },
    /// Create a default KDC config file if one does not exist.
    InitConfig,
    /// Print a dry-run deployment plan.
    DeployPlan,
    /// Check the local development environment.
    Doctor,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Doctor) => {
            let report = doctor::environment_check::run();
            println!("{}", report.render());
        }
        Some(Command::InitConfig) => {
            let path = config::paths::config_file();
            let settings = config::settings::Settings::load_or_default(&path)?;
            settings.save(&path)?;
            println!("Config ready at {}", path.display());
        }
        Some(Command::Scan) => {
            let state = startup::initialize(cli.project)?;
            println!("{}", state.project.summary());
            println!("{}", state.capabilities.summary());
        }
        Some(Command::Analysis) => {
            let state = startup::initialize(cli.project)?;
            let analysis = analyzer::ProjectAnalysis::from_context(
                &state.project,
                state.capabilities.clone(),
                state.runtime.clone(),
            );
            println!("{}", analysis.render());
        }
        Some(Command::Menus) => {
            let state = startup::initialize(cli.project)?;
            for item in &state.menus {
                println!("{}", item.display_line());
            }
        }
        Some(Command::Actions { query }) => {
            let state = startup::initialize(cli.project)?;
            let actions = query
                .as_deref()
                .map(|query| ui::command_palette::search_actions(&state.actions, query))
                .unwrap_or_else(|| state.actions.iter().collect());

            for action in actions {
                println!("{}", action.display_line());
            }
        }
        Some(Command::DeployPlan) => {
            let state = startup::initialize(cli.project)?;
            let plan = deploy::pipeline::plan(&state.capabilities, &state.runtime);
            println!("{}", plan.render());
        }
        None => {
            let state = startup::initialize(cli.project)?;
            ui::dashboard::run(state)?;
        }
    }

    Ok(())
}
