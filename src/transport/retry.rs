use crate::config::RetrySettings;
use crate::error::TransportError;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

pub async fn with_retry<F, Fut>(
    settings: &RetrySettings,
    operation: F,
) -> Result<(), TransportError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<(), TransportError>>,
{
    let mut backoff = ExponentialBackoff {
        initial_interval: Duration::from_millis(settings.initial_delay_ms),
        max_interval: Duration::from_millis(settings.max_delay_ms),
        multiplier: settings.multiplier,
        max_elapsed_time: None,
        ..Default::default()
    };

    let mut attempt = 0;

    loop {
        attempt += 1;

        match operation().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempt >= settings.max_attempts {
                    warn!(
                        attempt = attempt,
                        max_attempts = settings.max_attempts,
                        error = %e,
                        "Max retry attempts reached"
                    );
                    return Err(e);
                }

                if !e.is_retriable() {
                    warn!(error = %e, "Non-retriable error encountered");
                    return Err(e);
                }

                // Get the next backoff duration
                let wait_duration = match backoff.next_backoff() {
                    Some(duration) => duration,
                    None => {
                        warn!("Backoff exhausted");
                        return Err(e);
                    }
                };

                debug!(
                    attempt = attempt,
                    max_attempts = settings.max_attempts,
                    wait_ms = wait_duration.as_millis(),
                    error = %e,
                    "Retrying after error"
                );

                tokio::time::sleep(wait_duration).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let settings = RetrySettings::default();
        let result = with_retry(&settings, || async { Ok(()) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let settings = RetrySettings {
            max_attempts: 5,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = with_retry(&settings, || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(TransportError::Network("temporary error".to_string()))
                } else {
                    Ok(())
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let settings = RetrySettings {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let result = with_retry(&settings, || async {
            Err(TransportError::Network("persistent error".to_string()))
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_retry_on_non_retriable_error() {
        let settings = RetrySettings {
            max_attempts: 5,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = with_retry(&settings, || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err(TransportError::AuthFailed("invalid key".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
