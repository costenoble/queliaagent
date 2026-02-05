use crate::config::AgentConfig;
use crate::error::ServiceError;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_LABEL: &str = "com.agentquelia.agent";

pub fn install(user_level: bool) -> Result<(), ServiceError> {
    let plist_dir = if user_level {
        dirs::home_dir()
            .map(|h| h.join("Library/LaunchAgents"))
            .ok_or_else(|| ServiceError::InstallFailed("Cannot find home directory".to_string()))?
    } else {
        PathBuf::from("/Library/LaunchDaemons")
    };

    // Create directory if it doesn't exist
    fs::create_dir_all(&plist_dir)
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot create plist directory: {}", e)))?;

    let plist_path = plist_dir.join(format!("{}.plist", SERVICE_LABEL));

    // Get current executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot get executable path: {}", e)))?;

    // Get config path
    let config_path = AgentConfig::default_config_path()
        .unwrap_or_else(|| PathBuf::from("/etc/agentquelia/agent.toml"));

    // Get log directory
    let log_dir = AgentConfig::default_log_dir()
        .unwrap_or_else(|| PathBuf::from("/var/log/agentquelia"));

    // Create log directory
    fs::create_dir_all(&log_dir)
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot create log directory: {}", e)))?;

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>--config</string>
        <string>{config}</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_dir}/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/stderr.log</string>
    <key>WorkingDirectory</key>
    <string>/tmp</string>
</dict>
</plist>
"#,
        label = SERVICE_LABEL,
        exe = exe_path.display(),
        config = config_path.display(),
        log_dir = log_dir.display(),
    );

    fs::write(&plist_path, plist_content)
        .map_err(|e| ServiceError::InstallFailed(format!("Cannot write plist: {}", e)))?;

    // Set permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&plist_path)
            .map_err(|e| ServiceError::InstallFailed(e.to_string()))?
            .permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&plist_path, perms)
            .map_err(|e| ServiceError::InstallFailed(e.to_string()))?;
    }

    Ok(())
}

pub fn uninstall() -> Result<(), ServiceError> {
    // Try to unload first
    let _ = Command::new("launchctl")
        .args(["unload", &format!("~/Library/LaunchAgents/{}.plist", SERVICE_LABEL)])
        .output();

    // Remove user-level plist
    if let Some(home) = dirs::home_dir() {
        let user_plist = home.join(format!("Library/LaunchAgents/{}.plist", SERVICE_LABEL));
        if user_plist.exists() {
            fs::remove_file(&user_plist)
                .map_err(|e| ServiceError::UninstallFailed(e.to_string()))?;
        }
    }

    // Try to remove system-level plist (may fail without sudo)
    let system_plist = PathBuf::from(format!("/Library/LaunchDaemons/{}.plist", SERVICE_LABEL));
    if system_plist.exists() {
        let _ = fs::remove_file(&system_plist);
    }

    Ok(())
}

pub fn status() -> Result<String, ServiceError> {
    let output = Command::new("launchctl")
        .args(["list", SERVICE_LABEL])
        .output()
        .map_err(|e| ServiceError::NotFound)?;

    if output.status.success() {
        Ok("Running".to_string())
    } else {
        Ok("Not running".to_string())
    }
}
