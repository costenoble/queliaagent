use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::sources::{create_source, DataSource};
use crate::transport::{with_retry, SupabaseClient};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub struct Scheduler {
    source: Box<dyn DataSource>,
    transport: SupabaseClient,
    config: AgentConfig,
    shutdown_rx: broadcast::Receiver<()>,
}

impl Scheduler {
    pub fn new(
        config: AgentConfig,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<Self, AgentError> {
        let source = create_source(&config.source)?;
        let transport = SupabaseClient::new(&config.supabase)?;

        Ok(Self {
            source,
            transport,
            config,
            shutdown_rx,
        })
    }

    pub async fn run(&mut self) -> Result<(), AgentError> {
        info!(
            instance_id = %self.config.agent.instance_id,
            interval_secs = self.config.agent.polling_interval_secs,
            source = self.source.source_id(),
            "Starting scheduler"
        );

        let mut interval = tokio::time::interval(Duration::from_secs(
            self.config.agent.polling_interval_secs,
        ));

        // Don't burst on startup
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.poll_and_send().await;
                }
                _ = self.shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping scheduler");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn poll_and_send(&self) {
        info!(source = self.source.source_id(), "Polling data source");

        match self.source.read_value().await {
            Ok(reading) => {
                info!(
                    value = reading.value,
                    unit = %reading.unit,
                    source = %reading.source_id,
                    "Read value from source"
                );

                let api_key = self.config.poi.api_key.clone();
                let transport = &self.transport;
                let retry_settings = &self.config.retry;
                let value = reading.value;
                let unit = reading.unit.clone();

                let result = with_retry(retry_settings, || {
                    let api_key = api_key.clone();
                    let unit = unit.clone();
                    async move { transport.insert_live_data(&api_key, value, &unit).await }
                })
                .await;

                match result {
                    Ok(()) => {
                        info!(
                            value = reading.value,
                            unit = %reading.unit,
                            "Data sent successfully to Supabase"
                        );
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            value = reading.value,
                            "Failed to send data to Supabase after retries"
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    source = self.source.source_id(),
                    "Failed to read value from source"
                );
            }
        }
    }
}

pub struct AgentRunner {
    config: AgentConfig,
    shutdown_tx: broadcast::Sender<()>,
}

impl AgentRunner {
    pub fn new(config: AgentConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            shutdown_tx,
        }
    }

    pub async fn run(&self) -> Result<(), AgentError> {
        let shutdown_rx = self.shutdown_tx.subscribe();
        let mut scheduler = Scheduler::new(self.config.clone(), shutdown_rx)?;

        // Set up signal handlers
        let shutdown_tx = self.shutdown_tx.clone();
        tokio::spawn(async move {
            Self::wait_for_shutdown_signal().await;
            let _ = shutdown_tx.send(());
        });

        scheduler.run().await
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    #[cfg(unix)]
    async fn wait_for_shutdown_signal() {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT");
            }
        }
    }

    #[cfg(windows)]
    async fn wait_for_shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C");
    }
}
