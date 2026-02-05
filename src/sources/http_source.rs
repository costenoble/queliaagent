use super::{DataSource, Reading};
use crate::config::HttpSourceConfig;
use crate::error::SourceError;
use async_trait::async_trait;
use chrono::Utc;
use jsonpath_rust::JsonPathQuery;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use std::str::FromStr;
use std::time::Duration;

pub struct HttpSource {
    client: reqwest::Client,
    url: String,
    method: Method,
    json_path: String,
    unit: String,
    multiplier: f64,
    headers: HeaderMap,
    source_id: String,
}

impl HttpSource {
    pub fn new(config: &HttpSourceConfig) -> Result<Self, SourceError> {
        let method = Method::from_str(&config.method.to_uppercase())
            .map_err(|_| SourceError::ParseError(format!("Invalid HTTP method: {}", config.method)))?;

        // Build headers
        let mut headers = HeaderMap::new();
        for (key, value) in &config.headers {
            let header_name = HeaderName::from_str(key)
                .map_err(|_| SourceError::ParseError(format!("Invalid header name: {}", key)))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|_| SourceError::ParseError(format!("Invalid header value: {}", value)))?;
            headers.insert(header_name, header_value);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| SourceError::HttpError(e.to_string()))?;

        Ok(Self {
            client,
            url: config.url.clone(),
            method,
            json_path: config.json_path.clone(),
            unit: config.unit.clone(),
            multiplier: config.multiplier,
            headers,
            source_id: format!("http:{}", config.url),
        })
    }

    fn extract_value(&self, json: &serde_json::Value) -> Result<f64, SourceError> {
        let result = json
            .clone()
            .path(&self.json_path)
            .map_err(|e| SourceError::JsonError(format!("Invalid JSONPath: {}", e)))?;

        let value = match &result {
            serde_json::Value::Array(arr) => arr.first().cloned(),
            v => Some(v.clone()),
        };

        let value = value.ok_or_else(|| {
            SourceError::ValueNotFound(format!("No value found at path: {}", self.json_path))
        })?;

        match value {
            serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| {
                SourceError::InvalidValueType(format!("Number {} cannot be converted to f64", n))
            }),
            serde_json::Value::String(s) => s.trim().parse::<f64>().map_err(|_| {
                SourceError::InvalidValueType(format!("String '{}' is not a valid number", s))
            }),
            other => Err(SourceError::InvalidValueType(format!(
                "Expected number, got {:?}",
                other
            ))),
        }
    }
}

#[async_trait]
impl DataSource for HttpSource {
    async fn read_value(&self) -> Result<Reading, SourceError> {
        let request = self
            .client
            .request(self.method.clone(), &self.url)
            .headers(self.headers.clone());

        let response = request
            .send()
            .await
            .map_err(|e| SourceError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SourceError::HttpError(format!(
                "HTTP {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown error")
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SourceError::JsonError(e.to_string()))?;

        let raw_value = self.extract_value(&json)?;
        let value = raw_value * self.multiplier;

        Ok(Reading {
            value,
            unit: self.unit.clone(),
            timestamp: Utc::now(),
            source_id: self.source_id.clone(),
        })
    }

    fn source_id(&self) -> &str {
        &self.source_id
    }
}
