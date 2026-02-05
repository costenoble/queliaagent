use crate::config::SupabaseSettings;
use crate::error::TransportError;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Serialize;
use std::time::Duration;

pub struct SupabaseClient {
    client: reqwest::Client,
    base_url: String,
    anon_key: String,
    rpc_endpoint: String,
}

#[derive(Debug, Serialize)]
struct InsertLiveDataRequest {
    p_api_key: String,
    p_value: f64,
    p_unit: String,
}

impl SupabaseClient {
    pub fn new(settings: &SupabaseSettings) -> Result<Self, TransportError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(settings.timeout_secs))
            .build()
            .map_err(|e| TransportError::Network(e.to_string()))?;

        Ok(Self {
            client,
            base_url: settings.url.clone(),
            anon_key: settings.anon_key.clone(),
            rpc_endpoint: settings.rpc_endpoint.clone(),
        })
    }

    pub async fn insert_live_data(
        &self,
        api_key: &str,
        value: f64,
        unit: &str,
    ) -> Result<(), TransportError> {
        let url = format!("{}{}", self.base_url, self.rpc_endpoint);

        let body = InsertLiveDataRequest {
            p_api_key: api_key.to_string(),
            p_value: value,
            p_unit: unit.to_string(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "apikey",
            HeaderValue::from_str(&self.anon_key)
                .map_err(|e| TransportError::Network(format!("Invalid API key header: {}", e)))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TransportError::Timeout
                } else if e.is_connect() {
                    TransportError::Network(format!("Connection failed: {}", e))
                } else {
                    TransportError::Network(e.to_string())
                }
            })?;

        let status = response.status();

        if status.is_success() {
            return Ok(());
        }

        // Handle error responses
        let error_body = response.text().await.unwrap_or_default();

        match status.as_u16() {
            401 | 403 => Err(TransportError::AuthFailed(error_body)),
            429 => {
                // Try to extract retry-after header
                Err(TransportError::RateLimited(60))
            }
            500..=599 => Err(TransportError::ServerError {
                status: status.as_u16(),
                message: error_body,
            }),
            _ => Err(TransportError::InvalidResponse(format!(
                "HTTP {}: {}",
                status, error_body
            ))),
        }
    }
}
