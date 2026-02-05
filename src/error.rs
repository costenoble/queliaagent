use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Source error: {0}")]
    Source(#[from] SourceError),

    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Update error: {0}")]
    Update(#[from] UpdateError),

    #[error("Service error: {0}")]
    Service(#[from] ServiceError),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,

    #[error("Failed to read configuration: {0}")]
    ReadError(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read file: {0}")]
    ReadError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Value not found at path: {0}")]
    ValueNotFound(String),

    #[error("Invalid value type: expected number, got {0}")]
    InvalidValueType(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("CSV error: {0}")]
    CsvError(String),

    #[error("JSON error: {0}")]
    JsonError(String),
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Server error (status {status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("Request timeout")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

impl TransportError {
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::Network(_) | Self::RateLimited(_) | Self::Timeout => true,
            Self::ServerError { status, .. } => *status >= 500,
            _ => false,
        }
    }
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Failed to check for updates: {0}")]
    CheckFailed(String),

    #[error("Failed to download update: {0}")]
    DownloadFailed(String),

    #[error("Checksum verification failed: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Failed to install update: {0}")]
    InstallFailed(String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Failed to install service: {0}")]
    InstallFailed(String),

    #[error("Failed to uninstall service: {0}")]
    UninstallFailed(String),

    #[error("Failed to start service: {0}")]
    StartFailed(String),

    #[error("Failed to stop service: {0}")]
    StopFailed(String),

    #[error("Service not found")]
    NotFound,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
