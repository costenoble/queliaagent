use super::{DataSource, Reading};
use crate::config::CsvSourceConfig;
use crate::error::SourceError;
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;

pub struct CsvSource {
    path: PathBuf,
    value_field: String,
    unit: String,
    read_last_row: bool,
    delimiter: u8,
    skip_headers: usize,
    multiplier: f64,
    source_id: String,
}

impl CsvSource {
    pub fn new(config: &CsvSourceConfig) -> Result<Self, SourceError> {
        let delimiter = config
            .delimiter
            .chars()
            .next()
            .unwrap_or(',')
            .try_into()
            .map_err(|_| SourceError::ParseError("Invalid delimiter".to_string()))?;

        Ok(Self {
            path: config.path.clone(),
            value_field: config.value_field.clone(),
            unit: config.unit.clone(),
            read_last_row: config.read_last_row,
            delimiter,
            skip_headers: config.skip_headers,
            multiplier: config.multiplier,
            source_id: format!("csv:{}", config.path.display()),
        })
    }

    fn parse_value(&self, content: &str) -> Result<f64, SourceError> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(true)
            .from_reader(content.as_bytes());

        let headers = reader
            .headers()
            .map_err(|e| SourceError::CsvError(e.to_string()))?
            .clone();

        // Find the column index for the value field
        let column_idx = headers
            .iter()
            .position(|h| h == self.value_field)
            .or_else(|| self.value_field.parse::<usize>().ok())
            .ok_or_else(|| {
                SourceError::ValueNotFound(format!(
                    "Column '{}' not found in CSV",
                    self.value_field
                ))
            })?;

        // Read all records
        let records: Vec<csv::StringRecord> = reader
            .records()
            .skip(self.skip_headers)
            .filter_map(|r| r.ok())
            .collect();

        if records.is_empty() {
            return Err(SourceError::ValueNotFound(
                "No data rows found in CSV".to_string(),
            ));
        }

        // Get the appropriate row (last if read_last_row, first otherwise)
        let row = if self.read_last_row {
            records.last()
        } else {
            records.first()
        }
        .ok_or_else(|| SourceError::ValueNotFound("No rows found".to_string()))?;

        // Get the value from the column
        let value_str = row.get(column_idx).ok_or_else(|| {
            SourceError::ValueNotFound(format!("Column index {} out of bounds", column_idx))
        })?;

        // Parse the value as f64
        value_str
            .trim()
            .parse::<f64>()
            .map_err(|_| SourceError::InvalidValueType(format!("'{}'", value_str)))
    }
}

#[async_trait]
impl DataSource for CsvSource {
    async fn read_value(&self) -> Result<Reading, SourceError> {
        // Check if file exists
        if !self.path.exists() {
            return Err(SourceError::FileNotFound(
                self.path.display().to_string(),
            ));
        }

        // Read the file content
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| SourceError::ReadError(e.to_string()))?;

        let raw_value = self.parse_value(&content)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_csv_source_read_last_row() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timestamp,power_kw,status").unwrap();
        writeln!(file, "2024-01-01,100.5,ok").unwrap();
        writeln!(file, "2024-01-02,150.7,ok").unwrap();
        writeln!(file, "2024-01-03,200.3,ok").unwrap();

        let config = CsvSourceConfig {
            path: file.path().to_path_buf(),
            value_field: "power_kw".to_string(),
            unit: "kW".to_string(),
            read_last_row: true,
            delimiter: ",".to_string(),
            skip_headers: 0,
            multiplier: 1.0,
        };

        let source = CsvSource::new(&config).unwrap();
        let reading = source.read_value().await.unwrap();

        assert!((reading.value - 200.3).abs() < 0.001);
        assert_eq!(reading.unit, "kW");
    }

    #[tokio::test]
    async fn test_csv_source_read_first_row() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timestamp,power_kw,status").unwrap();
        writeln!(file, "2024-01-01,100.5,ok").unwrap();
        writeln!(file, "2024-01-02,150.7,ok").unwrap();

        let config = CsvSourceConfig {
            path: file.path().to_path_buf(),
            value_field: "power_kw".to_string(),
            unit: "kW".to_string(),
            read_last_row: false,
            delimiter: ",".to_string(),
            skip_headers: 0,
            multiplier: 1.0,
        };

        let source = CsvSource::new(&config).unwrap();
        let reading = source.read_value().await.unwrap();

        assert!((reading.value - 100.5).abs() < 0.001);
    }
}
