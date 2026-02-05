use crate::config::AgentConfig;
use crate::error::UpdateError;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, info};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct ReleaseManifest {
    version: String,
    assets: Assets,
}

#[derive(Debug, Deserialize)]
struct Assets {
    #[serde(rename = "macos-aarch64")]
    macos_aarch64: Option<Asset>,
    #[serde(rename = "macos-x86_64")]
    macos_x86_64: Option<Asset>,
    #[serde(rename = "windows-x86_64")]
    windows_x86_64: Option<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    url: String,
    checksum: String,
}

pub async fn check_and_update(force: bool) -> Result<bool, UpdateError> {
    // Load config to get update URL
    let config = AgentConfig::load(None).map_err(|e| UpdateError::CheckFailed(e.to_string()))?;

    if !config.update.enabled && !force {
        info!("Updates are disabled in configuration");
        return Ok(false);
    }

    if config.update.update_url.is_empty() {
        return Err(UpdateError::CheckFailed(
            "No update URL configured".to_string(),
        ));
    }

    // Fetch the release manifest
    let manifest = fetch_manifest(&config.update.update_url).await?;

    // Parse versions
    let current = Version::parse(CURRENT_VERSION)
        .map_err(|e| UpdateError::InvalidVersion(format!("Current: {}", e)))?;
    let latest = Version::parse(&manifest.version)
        .map_err(|e| UpdateError::InvalidVersion(format!("Latest: {}", e)))?;

    info!(
        current = %current,
        latest = %latest,
        "Version check complete"
    );

    if latest <= current && !force {
        return Ok(false);
    }

    info!("New version available: {} -> {}", current, latest);

    // Get the appropriate asset for this platform
    let asset = get_platform_asset(&manifest.assets)?;

    // Download and install
    download_and_install(&asset).await?;

    Ok(true)
}

async fn fetch_manifest(url: &str) -> Result<ReleaseManifest, UpdateError> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| UpdateError::CheckFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(UpdateError::CheckFailed(format!(
            "HTTP {}",
            response.status()
        )));
    }

    response
        .json()
        .await
        .map_err(|e| UpdateError::CheckFailed(e.to_string()))
}

fn get_platform_asset(assets: &Assets) -> Result<&Asset, UpdateError> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        assets
            .macos_aarch64
            .as_ref()
            .ok_or_else(|| UpdateError::CheckFailed("No macOS ARM64 asset found".to_string()))
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        assets
            .macos_x86_64
            .as_ref()
            .ok_or_else(|| UpdateError::CheckFailed("No macOS x86_64 asset found".to_string()))
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        assets
            .windows_x86_64
            .as_ref()
            .ok_or_else(|| UpdateError::CheckFailed("No Windows x86_64 asset found".to_string()))
    }

    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    {
        Err(UpdateError::CheckFailed(
            "Unsupported platform for auto-update".to_string(),
        ))
    }
}

async fn download_and_install(asset: &Asset) -> Result<(), UpdateError> {
    info!(url = %asset.url, "Downloading update...");

    let client = reqwest::Client::new();
    let response = client
        .get(&asset.url)
        .send()
        .await
        .map_err(|e| UpdateError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(UpdateError::DownloadFailed(format!(
            "HTTP {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| UpdateError::DownloadFailed(e.to_string()))?;

    // Verify checksum
    verify_checksum(&bytes, &asset.checksum)?;

    // Write to temp file
    let temp_path = get_temp_path()?;
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| UpdateError::InstallFailed(format!("Failed to create temp file: {}", e)))?;

    file.write_all(&bytes)
        .map_err(|e| UpdateError::InstallFailed(format!("Failed to write temp file: {}", e)))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_path)
            .map_err(|e| UpdateError::InstallFailed(e.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_path, perms)
            .map_err(|e| UpdateError::InstallFailed(e.to_string()))?;
    }

    // Replace current binary
    info!("Installing update...");
    self_replace::self_replace(&temp_path)
        .map_err(|e| UpdateError::InstallFailed(e.to_string()))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    info!("Update installed successfully");
    Ok(())
}

fn verify_checksum(data: &[u8], expected: &str) -> Result<(), UpdateError> {
    // Expected format: "sha256:abc123..."
    let expected_hash = expected
        .strip_prefix("sha256:")
        .unwrap_or(expected)
        .to_lowercase();

    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual_hash = format!("{:x}", hasher.finalize());

    if actual_hash != expected_hash {
        return Err(UpdateError::ChecksumMismatch {
            expected: expected_hash,
            actual: actual_hash,
        });
    }

    debug!("Checksum verified: {}", actual_hash);
    Ok(())
}

fn get_temp_path() -> Result<PathBuf, UpdateError> {
    let temp_dir = env::temp_dir();
    let filename = format!("agentquelia-update-{}", uuid::Uuid::new_v4());

    #[cfg(windows)]
    let filename = format!("{}.exe", filename);

    Ok(temp_dir.join(filename))
}
