pub mod csv_source;
pub mod http_source;
pub mod json_source;

pub use csv_source::CsvSource;
pub use http_source::HttpSource;
pub use json_source::JsonSource;

use crate::config::{SourceConfig, SourceType};
use crate::error::SourceError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Reading {
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub source_id: String,
}

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn read_value(&self) -> Result<Reading, SourceError>;
    fn source_id(&self) -> &str;
}

pub fn create_source(config: &SourceConfig) -> Result<Box<dyn DataSource>, SourceError> {
    match config.source_type {
        SourceType::Csv => {
            let csv_config = config
                .csv
                .as_ref()
                .ok_or_else(|| SourceError::ParseError("Missing CSV configuration".to_string()))?;
            Ok(Box::new(CsvSource::new(csv_config)?))
        }
        SourceType::Json => {
            let json_config = config
                .json
                .as_ref()
                .ok_or_else(|| SourceError::ParseError("Missing JSON configuration".to_string()))?;
            Ok(Box::new(JsonSource::new(json_config)?))
        }
        SourceType::Http => {
            let http_config = config
                .http
                .as_ref()
                .ok_or_else(|| SourceError::ParseError("Missing HTTP configuration".to_string()))?;
            Ok(Box::new(HttpSource::new(http_config)?))
        }
    }
}
