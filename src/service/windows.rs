use crate::config::AgentConfig;
use crate::error::ServiceError;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "agentquelia";
const SERVICE_DISPLAY_NAME: &str = "Agentquelia Power Data Agent";

pub fn install() -> Result<(), ServiceError> {
    // Get current executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot get executable path: {}", e)))?;

    // Get config path
    let config_path = AgentConfig::default_config_path()
        .unwrap_or_else(|| PathBuf::from("C:\\ProgramData\\agentquelia\\agent.toml"));

    let bin_path = format!(
        "\"{}\" run --config \"{}\"",
        exe_path.display(),
        config_path.display()
    );

    // Create the service using sc.exe
    let output = Command::new("sc")
        .args([
            "create",
            SERVICE_NAME,
            &format!("binPath={}", bin_path),
            &format!("DisplayName={}", SERVICE_DISPLAY_NAME),
            "start=auto",
        ])
        .output()
        .map_err(|e| ServiceError::InstallFailed(format!("Failed to run sc: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ServiceError::InstallFailed(format!(
            "sc create failed: {}",
            stderr
        )));
    }

    // Set description
    let _ = Command::new("sc")
        .args([
            "description",
            SERVICE_NAME,
            "Agentquelia power data collection agent",
        ])
        .output();

    Ok(())
}

pub fn uninstall() -> Result<(), ServiceError> {
    // Stop the service first
    let _ = Command::new("sc").args(["stop", SERVICE_NAME]).output();

    // Delete the service
    let output = Command::new("sc")
        .args(["delete", SERVICE_NAME])
        .output()
        .map_err(|e| ServiceError::UninstallFailed(format!("Failed to run sc: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ServiceError::UninstallFailed(format!(
            "sc delete failed: {}",
            stderr
        )));
    }

    Ok(())
}

pub fn status() -> Result<String, ServiceError> {
    let output = Command::new("sc")
        .args(["query", SERVICE_NAME])
        .output()
        .map_err(|_| ServiceError::NotFound)?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("RUNNING") {
            Ok("Running".to_string())
        } else if stdout.contains("STOPPED") {
            Ok("Stopped".to_string())
        } else {
            Ok("Unknown".to_string())
        }
    } else {
        Ok("Not installed".to_string())
    }
}
