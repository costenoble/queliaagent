use super::{DataSource, Reading};
use crate::config::JsonSourceConfig;
use crate::error::SourceError;
use async_trait::async_trait;
use chrono::Utc;
use jsonpath_rust::JsonPathQuery;
use std::path::PathBuf;

pub struct JsonSource {
    path: PathBuf,
    json_path: String,
    unit: String,
    multiplier: f64,
    source_id: String,
}

impl JsonSource {
    pub fn new(config: &JsonSourceConfig) -> Result<Self, SourceError> {
        Ok(Self {
            path: config.path.clone(),
            json_path: config.json_path.clone(),
            unit: config.unit.clone(),
            multiplier: config.multiplier,
            source_id: format!("json:{}", config.path.display()),
        })
    }

    fn extract_value(&self, json: &serde_json::Value) -> Result<f64, SourceError> {
        let result = json
            .clone()
            .path(&self.json_path)
            .map_err(|e| SourceError::JsonError(format!("Invalid JSONPath: {}", e)))?;

        // The result is a Value that could be an array or single value
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
impl DataSource for JsonSource {
    async fn read_value(&self) -> Result<Reading, SourceError> {
        if !self.path.exists() {
            return Err(SourceError::FileNotFound(
                self.path.display().to_string(),
            ));
        }

        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| SourceError::ReadError(e.to_string()))?;

        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| SourceError::JsonError(e.to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_json_source_simple_path() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
            "power": 123.45,
            "unit": "kW"
        }}"#
        )
        .unwrap();

        let config = JsonSourceConfig {
            path: file.path().to_path_buf(),
            json_path: "$.power".to_string(),
            unit: "kW".to_string(),
            multiplier: 1.0,
        };

        let source = JsonSource::new(&config).unwrap();
        let reading = source.read_value().await.unwrap();

        assert!((reading.value - 123.45).abs() < 0.001);
        assert_eq!(reading.unit, "kW");
    }

    #[tokio::test]
    async fn test_json_source_nested_path() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
            "meters": [
                {{"id": 1, "reading": 100.0}},
                {{"id": 2, "reading": 200.5}}
            ]
        }}"#
        )
        .unwrap();

        let config = JsonSourceConfig {
            path: file.path().to_path_buf(),
            json_path: "$.meters[0].reading".to_string(),
            unit: "MW".to_string(),
            multiplier: 1.0,
        };

        let source = JsonSource::new(&config).unwrap();
        let reading = source.read_value().await.unwrap();

        assert!((reading.value - 100.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_json_source_with_multiplier() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
            "power_kw": 1000.0
        }}"#
        )
        .unwrap();

        // Convertir kW en MW (multiplier par 0.001)
        let config = JsonSourceConfig {
            path: file.path().to_path_buf(),
            json_path: "$.power_kw".to_string(),
            unit: "MW".to_string(),
            multiplier: 0.001,
        };

        let source = JsonSource::new(&config).unwrap();
        let reading = source.read_value().await.unwrap();

        // 1000 kW * 0.001 = 1 MW
        assert!((reading.value - 1.0).abs() < 0.0001);
        assert_eq!(reading.unit, "MW");
    }
}
