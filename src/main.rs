mod cli;
mod config;
mod error;
mod logging;
mod scheduler;
mod service;
mod sources;
mod transport;
mod update;

use cli::{Cli, Command};
use config::AgentConfig;
use error::AgentError;
use scheduler::AgentRunner;
use std::process::ExitCode;
use tracing::{error, info};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse_args();

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), AgentError> {
    match cli.command {
        Command::Run => run_agent(cli).await,
        Command::Install { user } => install_service(user).await,
        Command::Uninstall => uninstall_service().await,
        Command::Update { force } => run_update(force).await,
        Command::Config { show_secrets } => show_config(cli, show_secrets),
        Command::Validate => validate_config(cli),
        Command::Status => show_status().await,
    }
}

async fn run_agent(cli: Cli) -> Result<(), AgentError> {
    // Load configuration
    let config = AgentConfig::load(cli.config.as_deref())?;

    // Initialize logging
    let _guard = logging::init_logging(&config.logging);

    info!(
        version = env!("CARGO_PKG_VERSION"),
        instance_id = %config.agent.instance_id,
        "Starting Agentquelia"
    );

    // Create and run the agent
    let runner = AgentRunner::new(config);
    runner.run().await?;

    info!("Agentquelia stopped");
    Ok(())
}

async fn install_service(user_level: bool) -> Result<(), AgentError> {
    #[cfg(target_os = "macos")]
    {
        service::macos::install(user_level)?;
        println!("Service installed successfully");
        println!("Start with: launchctl load ~/Library/LaunchAgents/com.agentquelia.agent.plist");
    }

    #[cfg(target_os = "windows")]
    {
        service::windows::install()?;
        println!("Service installed successfully");
        println!("Start with: sc start agentquelia");
    }

    #[cfg(target_os = "linux")]
    {
        service::linux::install(user_level)?;
        println!("Service installed successfully");
        println!("Start with: sudo systemctl start agentquelia");
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        eprintln!("Service installation not supported on this platform");
    }

    Ok(())
}

async fn uninstall_service() -> Result<(), AgentError> {
    #[cfg(target_os = "macos")]
    {
        service::macos::uninstall()?;
        println!("Service uninstalled successfully");
    }

    #[cfg(target_os = "windows")]
    {
        service::windows::uninstall()?;
        println!("Service uninstalled successfully");
    }

    #[cfg(target_os = "linux")]
    {
        service::linux::uninstall()?;
        println!("Service uninstalled successfully");
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        eprintln!("Service uninstallation not supported on this platform");
    }

    Ok(())
}

async fn run_update(force: bool) -> Result<(), AgentError> {
    logging::init_console_only("info");

    info!("Checking for updates...");

    match update::check_and_update(force).await {
        Ok(updated) => {
            if updated {
                info!("Update installed successfully. Please restart the agent.");
            } else {
                info!("Already running the latest version.");
            }
        }
        Err(e) => {
            error!(error = %e, "Update failed");
            return Err(e.into());
        }
    }

    Ok(())
}

fn show_config(cli: Cli, show_secrets: bool) -> Result<(), AgentError> {
    let config = AgentConfig::load(cli.config.as_deref())?;

    println!("Configuration:");
    println!("  Instance ID: {}", config.agent.instance_id);
    println!(
        "  Polling interval: {} seconds",
        config.agent.polling_interval_secs
    );
    println!();

    println!("POI:");
    if show_secrets {
        println!("  API Key: {}", config.poi.api_key);
    } else {
        println!("  API Key: {}...", &config.poi.api_key[..12.min(config.poi.api_key.len())]);
    }
    println!();

    println!("Supabase:");
    println!("  URL: {}", config.supabase.url);
    println!("  RPC Endpoint: {}", config.supabase.rpc_endpoint);
    if show_secrets {
        println!("  Anon Key: {}", config.supabase.anon_key);
    } else {
        println!("  Anon Key: [REDACTED]");
    }
    println!();

    println!("Source:");
    println!("  Type: {:?}", config.source.source_type);
    match config.source.source_type {
        config::SourceType::Csv => {
            if let Some(csv) = &config.source.csv {
                println!("  Path: {}", csv.path.display());
                println!("  Value Field: {}", csv.value_field);
                println!("  Unit: {}", csv.unit);
            }
        }
        config::SourceType::Json => {
            if let Some(json) = &config.source.json {
                println!("  Path: {}", json.path.display());
                println!("  JSON Path: {}", json.json_path);
                println!("  Unit: {}", json.unit);
            }
        }
        config::SourceType::Http => {
            if let Some(http) = &config.source.http {
                println!("  URL: {}", http.url);
                println!("  Method: {}", http.method);
                println!("  JSON Path: {}", http.json_path);
                println!("  Unit: {}", http.unit);
            }
        }
    }
    println!();

    println!("Logging:");
    println!("  Level: {}", config.logging.level);
    println!("  Rotation: {:?}", config.logging.rotation);
    println!("  Console output: {}", config.logging.console_output);
    println!();

    println!("Update:");
    println!("  Enabled: {}", config.update.enabled);
    if !config.update.update_url.is_empty() {
        println!("  URL: {}", config.update.update_url);
    }

    Ok(())
}

fn validate_config(cli: Cli) -> Result<(), AgentError> {
    match AgentConfig::load(cli.config.as_deref()) {
        Ok(_) => {
            println!("Configuration is valid.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            Err(e.into())
        }
    }
}

async fn show_status() -> Result<(), AgentError> {
    #[cfg(target_os = "macos")]
    {
        let status = service::macos::status()?;
        println!("Service status: {}", status);
    }

    #[cfg(target_os = "windows")]
    {
        let status = service::windows::status()?;
        println!("Service status: {}", status);
    }

    #[cfg(target_os = "linux")]
    {
        let status = service::linux::status()?;
        println!("Service status: {}", status);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        println!("Service status not available on this platform");
    }

    Ok(())
}
