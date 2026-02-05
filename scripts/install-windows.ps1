#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Agentquelia Windows Installation Script
.DESCRIPTION
    Installs Agentquelia as a Windows service
.EXAMPLE
    .\install-windows.ps1
    .\install-windows.ps1 -BinaryUrl "https://example.com/agentquelia.exe"
#>

param(
    [string]$InstallDir = "$env:ProgramFiles\agentquelia",
    [string]$ConfigDir = "$env:APPDATA\agentquelia",
    [string]$LogDir = "$env:LOCALAPPDATA\agentquelia\logs",
    [string]$BinaryUrl = ""
)

$ErrorActionPreference = "Stop"

Write-Host "Agentquelia Windows Installation" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host ""

# Determine binary source
$binaryPath = $null
if (Test-Path ".\target\release\agentquelia.exe") {
    $binaryPath = ".\target\release\agentquelia.exe"
    Write-Host "Installing from local build..."
} elseif ($BinaryUrl) {
    Write-Host "Downloading from $BinaryUrl..."
    $binaryPath = "$env:TEMP\agentquelia.exe"
    Invoke-WebRequest -Uri $BinaryUrl -OutFile $binaryPath
} else {
    Write-Host "Error: No binary found. Either build locally or provide -BinaryUrl" -ForegroundColor Red
    Write-Host "To build locally: cargo build --release"
    exit 1
}

# Create directories
Write-Host "Creating directories..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

# Install binary
Write-Host "Installing binary to $InstallDir..."
Copy-Item $binaryPath "$InstallDir\agentquelia.exe" -Force

# Add to PATH if not already there
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Host "Adding to system PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "Machine")
}

# Create example config if none exists
$configFile = "$ConfigDir\agent.toml"
if (-not (Test-Path $configFile)) {
    Write-Host "Creating example configuration..."

    if (Test-Path ".\config\agent.example.toml") {
        Copy-Item ".\config\agent.example.toml" $configFile
    } else {
        @"
[agent]
instance_id = "poi-001"
polling_interval_secs = 60

[poi]
api_key = "`${AGENTQUELIA_POI_KEY}"

[supabase]
url = "https://msqisigttxosvnxfhfdn.supabase.co"
anon_key = "`${SUPABASE_ANON_KEY}"

[source]
type = "csv"

[source.csv]
path = "C:\\data\\readings.csv"
value_field = "power_kw"
unit = "kW"

[logging]
level = "info"
"@ | Out-File -FilePath $configFile -Encoding utf8
    }

    Write-Host "IMPORTANT: Edit $configFile with your settings" -ForegroundColor Yellow
}

# Install as service
Write-Host "Installing Windows service..."
& "$InstallDir\agentquelia.exe" install 2>$null

Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Edit your configuration: $configFile"
Write-Host "2. Set environment variables:"
Write-Host "   setx AGENTQUELIA_POI_KEY 'your_poi_key'"
Write-Host "   setx SUPABASE_ANON_KEY 'your_supabase_key'"
Write-Host "3. Test the agent: agentquelia run --config `"$configFile`""
Write-Host "4. Start the service: sc start agentquelia"
Write-Host ""
Write-Host "Logs: $LogDir"
