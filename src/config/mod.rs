pub mod schema;

pub use schema::*;

use crate::error::ConfigError;
use std::path::{Path, PathBuf};

impl AgentConfig {
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        let config_path = path
            .map(PathBuf::from)
            .or_else(Self::default_config_path)
            .ok_or(ConfigError::NotFound)?;

        if !config_path.exists() {
            return Err(ConfigError::NotFound);
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;

        // Expand environment variables in the content
        let expanded = expand_env_vars(&content);

        let config: AgentConfig =
            toml::from_str(&expanded).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    pub fn default_config_path() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            dirs::config_dir().map(|p| p.join("agentquelia/agent.toml"))
        }

        #[cfg(target_os = "windows")]
        {
            dirs::config_dir().map(|p| p.join("agentquelia\\agent.toml"))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            dirs::config_dir().map(|p| p.join("agentquelia/agent.toml"))
        }
    }

    pub fn default_log_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|p| p.join("Library/Logs/agentquelia"))
        }

        #[cfg(target_os = "windows")]
        {
            dirs::data_local_dir().map(|p| p.join("agentquelia\\logs"))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            dirs::data_local_dir().map(|p| p.join("agentquelia/logs"))
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // Validate instance_id
        if self.agent.instance_id.is_empty() {
            return Err(ConfigError::ValidationError(
                "agent.instance_id cannot be empty".to_string(),
            ));
        }

        // Validate polling interval
        if self.agent.polling_interval_secs == 0 {
            return Err(ConfigError::ValidationError(
                "agent.polling_interval_secs must be greater than 0".to_string(),
            ));
        }

        // Validate POI API key
        if self.poi.api_key.is_empty() {
            return Err(ConfigError::ValidationError(
                "poi.api_key cannot be empty".to_string(),
            ));
        }

        // Validate Supabase settings
        if self.supabase.url.is_empty() {
            return Err(ConfigError::ValidationError(
                "supabase.url cannot be empty".to_string(),
            ));
        }

        if self.supabase.anon_key.is_empty() {
            return Err(ConfigError::ValidationError(
                "supabase.anon_key cannot be empty".to_string(),
            ));
        }

        // Validate source configuration
        match self.source.source_type {
            SourceType::Csv => {
                if self.source.csv.is_none() {
                    return Err(ConfigError::ValidationError(
                        "source.csv is required when type is 'csv'".to_string(),
                    ));
                }
                let csv = self.source.csv.as_ref().unwrap();
                if csv.value_field.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.csv.value_field cannot be empty".to_string(),
                    ));
                }
                if csv.unit.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.csv.unit cannot be empty".to_string(),
                    ));
                }
            }
            SourceType::Json => {
                if self.source.json.is_none() {
                    return Err(ConfigError::ValidationError(
                        "source.json is required when type is 'json'".to_string(),
                    ));
                }
                let json = self.source.json.as_ref().unwrap();
                if json.json_path.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.json.json_path cannot be empty".to_string(),
                    ));
                }
                if json.unit.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.json.unit cannot be empty".to_string(),
                    ));
                }
            }
            SourceType::Http => {
                if self.source.http.is_none() {
                    return Err(ConfigError::ValidationError(
                        "source.http is required when type is 'http'".to_string(),
                    ));
                }
                let http = self.source.http.as_ref().unwrap();
                if http.url.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.http.url cannot be empty".to_string(),
                    ));
                }
                if http.json_path.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.http.json_path cannot be empty".to_string(),
                    ));
                }
                if http.unit.is_empty() {
                    return Err(ConfigError::ValidationError(
                        "source.http.unit cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

fn expand_env_vars(content: &str) -> String {
    let mut result = content.to_string();

    // Find all ${VAR_NAME} patterns and expand them
    let re = regex_lite::Regex::new(r"\$\{([^}]+)\}").unwrap();

    for cap in re.captures_iter(content) {
        let full_match = cap.get(0).unwrap().as_str();
        let var_name = cap.get(1).unwrap().as_str();

        if let Ok(value) = std::env::var(var_name) {
            result = result.replace(full_match, &value);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars() {
        std::env::set_var("TEST_VAR", "test_value");
        let input = "key = \"${TEST_VAR}\"";
        let result = expand_env_vars(input);
        assert_eq!(result, "key = \"test_value\"");
        std::env::remove_var("TEST_VAR");
    }
}
