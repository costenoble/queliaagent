use crate::config::{AgentConfig, LogRotation, LoggingSettings};
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_logging(config: &LoggingSettings) -> Option<WorkerGuard> {
    let log_dir = config
        .directory
        .clone()
        .or_else(default_log_dir)
        .unwrap_or_else(|| PathBuf::from("."));

    // Create log directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    let file_appender = match config.rotation {
        LogRotation::Daily => tracing_appender::rolling::daily(&log_dir, "agentquelia.log"),
        LogRotation::Hourly => tracing_appender::rolling::hourly(&log_dir, "agentquelia.log"),
        LogRotation::Never => tracing_appender::rolling::never(&log_dir, "agentquelia.log"),
    };

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    if config.console_output {
        let console_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_target(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(console_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    Some(guard)
}

pub fn init_console_only(level: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_target(true),
        )
        .init();
}

fn default_log_dir() -> Option<PathBuf> {
    AgentConfig::default_log_dir()
}
