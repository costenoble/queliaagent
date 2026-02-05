use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    pub agent: AgentSettings,
    pub poi: PoiSettings,
    pub supabase: SupabaseSettings,
    pub source: SourceConfig,
    #[serde(default)]
    pub logging: LoggingSettings,
    #[serde(default)]
    pub update: UpdateSettings,
    #[serde(default)]
    pub retry: RetrySettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentSettings {
    pub instance_id: String,
    #[serde(default = "default_polling_interval")]
    pub polling_interval_secs: u64,
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoiSettings {
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupabaseSettings {
    pub url: String,
    pub anon_key: String,
    #[serde(default = "default_rpc_endpoint")]
    pub rpc_endpoint: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceConfig {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub csv: Option<CsvSourceConfig>,
    pub json: Option<JsonSourceConfig>,
    pub http: Option<HttpSourceConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Csv,
    Json,
    Http,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CsvSourceConfig {
    pub path: PathBuf,
    pub value_field: String,
    pub unit: String,
    #[serde(default = "default_true")]
    pub read_last_row: bool,
    #[serde(default = "default_delimiter")]
    pub delimiter: String,
    #[serde(default)]
    pub skip_headers: usize,
    #[serde(default = "default_multiplier_one")]
    pub multiplier: f64,  // Ex: 0.001 pour convertir kW en MW
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonSourceConfig {
    pub path: PathBuf,
    pub json_path: String,
    pub unit: String,
    #[serde(default = "default_multiplier_one")]
    pub multiplier: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpSourceConfig {
    pub url: String,
    #[serde(default = "default_http_method")]
    pub method: String,
    pub json_path: String,
    pub unit: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default_http_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_multiplier_one")]
    pub multiplier: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingSettings {
    #[serde(default = "default_log_level")]
    pub level: String,
    pub directory: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub console_output: bool,
    #[serde(default)]
    pub rotation: LogRotation,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            directory: None,
            console_output: true,
            rotation: LogRotation::default(),
            max_files: default_max_files(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    #[default]
    Daily,
    Hourly,
    Never,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
    #[serde(default)]
    pub update_url: String,
    #[serde(default = "default_channel")]
    pub channel: String,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            check_interval_hours: default_check_interval(),
            update_url: String::new(),
            channel: default_channel(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetrySettings {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            initial_delay_ms: default_initial_delay(),
            max_delay_ms: default_max_delay(),
            multiplier: default_multiplier(),
        }
    }
}

// Default value functions
fn default_polling_interval() -> u64 {
    60
}

fn default_rpc_endpoint() -> String {
    "/rest/v1/rpc/insert_live_data".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

fn default_delimiter() -> String {
    ",".to_string()
}

fn default_http_method() -> String {
    "GET".to_string()
}

fn default_http_timeout() -> u64 {
    10
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_files() -> usize {
    7
}

fn default_check_interval() -> u64 {
    24
}

fn default_channel() -> String {
    "stable".to_string()
}

fn default_max_attempts() -> u32 {
    5
}

fn default_initial_delay() -> u64 {
    1000
}

fn default_max_delay() -> u64 {
    60000
}

fn default_multiplier() -> f64 {
    2.0
}

fn default_multiplier_one() -> f64 {
    1.0
}
