use crate::config::AgentConfig;
use crate::error::ServiceError;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "agentquelia";

pub fn install(_user_level: bool) -> Result<(), ServiceError> {
    let service_file = PathBuf::from("/etc/systemd/system/agentquelia.service");

    // Get current executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot get executable path: {}", e)))?;

    // Get config path
    let config_path = PathBuf::from("/etc/agentquelia/agent.toml");

    // Create config directory
    fs::create_dir_all("/etc/agentquelia")
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot create config directory: {}", e)))?;

    let service_content = format!(
        r#"[Unit]
Description=Agentquelia Power Data Collection Agent
After=network.target

[Service]
Type=simple
ExecStart={exe} run
Environment=AGENTQUELIA_CONFIG={config}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=agentquelia

[Install]
WantedBy=multi-user.target
"#,
        exe = exe_path.display(),
        config = config_path.display(),
    );

    fs::write(&service_file, service_content)
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot write service file: {}", e)))?;

    // Reload systemd
    let _ = Command::new("systemctl")
        .args(["daemon-reload"])
        .output();

    // Enable service
    let _ = Command::new("systemctl")
        .args(["enable", SERVICE_NAME])
        .output();

    Ok(())
}

pub fn uninstall() -> Result<(), ServiceError> {
    // Stop service
    let _ = Command::new("systemctl")
        .args(["stop", SERVICE_NAME])
        .output();

    // Disable service
    let _ = Command::new("systemctl")
        .args(["disable", SERVICE_NAME])
        .output();

    // Remove service file
    let service_file = PathBuf::from("/etc/systemd/system/agentquelia.service");
    if service_file.exists() {
        fs::remove_file(&service_file)
            .map_err(|e| ServiceError::UninstallFailed(e.to_string()))?;
    }

    // Reload systemd
    let _ = Command::new("systemctl")
        .args(["daemon-reload"])
        .output();

    Ok(())
}

pub fn status() -> Result<String, ServiceError> {
    let output = Command::new("systemctl")
        .args(["is-active", SERVICE_NAME])
        .output()
        .map_err(|_| ServiceError::NotFound)?;

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status)
}
