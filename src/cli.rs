use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "agentquelia")]
#[command(author = "Agentquelia Team")]
#[command(version)]
#[command(about = "Cross-platform power data collection agent", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, env = "AGENTQUELIA_CONFIG")]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the agent in foreground
    Run,

    /// Install as system service
    Install {
        /// Install as user-level service (macOS only)
        #[arg(long)]
        user: bool,
    },

    /// Uninstall system service
    Uninstall,

    /// Check for and apply updates
    Update {
        /// Force update even if on latest version
        #[arg(long)]
        force: bool,
    },

    /// Show current configuration
    Config {
        /// Show full configuration including secrets
        #[arg(long)]
        show_secrets: bool,
    },

    /// Validate configuration file
    Validate,

    /// Show service status
    Status,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
