use std::path::PathBuf;

use clap::{Parser, Subcommand};
use kdc::{
    app::startup::{self, StartupOptions},
    config, deploy, doctor,
    project::analyzer,
    ui,
};

#[derive(Debug, Parser)]
#[command(name = "kdc")]
#[command(about = "Kubernetes Docker Commander")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(short, long, value_name = "PATH", default_value = ".")]
    project: PathBuf,

    #[arg(long)]
    first_launch: bool,
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
    /// Execute the deployment pipeline.
    Deploy {
        /// Target environment (development, staging, production).
        #[arg(short, long, default_value = "development")]
        environment: String,
    },
    /// Check the local development environment.
    Doctor {
        /// Run the full set of checks including registry and OS hints.
        #[arg(long)]
        full: bool,
        /// Export the doctor report as JSON.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Doctor { full, json }) => {
            run_doctor(full, json)?;
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
        Some(Command::Deploy { environment }) => {
            run_deploy(cli.project, &environment)?;
        }
        None => {
            let state = startup::initialize_with_options(
                cli.project,
                StartupOptions {
                    force_first_launch: cli.first_launch,
                },
            )?;
            ui::dashboard::run(state)?;
        }
    }

    Ok(())
}

fn run_doctor(full: bool, json: bool) -> anyhow::Result<()> {
    let report = if full {
        let settings = config::settings::Settings::load_or_default(&config::paths::config_file())?;
        doctor::environment_check::run_full(settings.registry.as_deref())
    } else {
        doctor::environment_check::run()
    };

    if json {
        println!("{}", report.export_json());
        // Also save to the doctor report file.
        let report_path = config::paths::doctor_report_file();
        if let Some(parent) = report_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&report_path, report.export_json())?;
        println!("Report saved to {}", report_path.display());
    } else {
        println!("{}", report.render());
        println!("\n{}", report.summary_line());
    }
    Ok(())
}

fn run_deploy(project: std::path::PathBuf, environment: &str) -> anyhow::Result<()> {
    let state = startup::initialize(project)?;
    let plan = deploy::pipeline::plan(&state.capabilities, &state.runtime);

    if !plan.ready() {
        let mut msg = "Deployment plan has blockers:\n".to_string();
        for blocker in &plan.blockers {
            msg.push_str(&format!("  - {}\n", blocker));
        }
        anyhow::bail!("{}", msg.trim_end());
    }

    println!("Executing deployment pipeline...\n");
    let execution = deploy::pipeline::execute_pipeline(
        &plan,
        &state.project,
        &state.capabilities,
        environment,
    )?;
    println!("{}", execution.render());

    // Record the deployment in history.
    let env = deploy::environments::from_string(environment);
    let history_path = config::paths::deploy_history_file();
    let mut history = deploy::history::DeploymentHistory::load_or_default(&history_path)?;
    history.record(deploy::history::DeploymentRecord {
        timestamp: chrono_timestamp(),
        environment: env.to_string(),
        image_tag: format!(
            "{}:latest",
            state.project.name.to_lowercase().replace(' ', "-")
        ),
        success: execution.overall_success,
        steps_completed: execution.results.iter().filter(|r| r.success).count(),
        steps_total: execution.results.len(),
        duration_secs: execution.total_duration_secs(),
        message: if execution.overall_success {
            "All steps completed".to_string()
        } else {
            execution
                .results
                .iter()
                .find(|r| !r.success)
                .map(|r| r.message.clone())
                .unwrap_or_else(|| "Unknown failure".to_string())
        },
    });
    history.save(&history_path)?;
    println!("\nDeployment recorded in {}", history_path.display());
    Ok(())
}

/// Generate an ISO 8601 timestamp string.
fn chrono_timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    if let Ok(formatted) = now.format(&time::format_description::well_known::Rfc3339) {
        return formatted;
    }
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("epoch:{}", duration.as_secs())
}
