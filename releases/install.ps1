# ============================================
# Agentquelia Installer for Windows
# ============================================
# Usage: irm https://URL/install.ps1 | iex
# Or:    .\install.ps1 -PoiKey "sk_live_xxx" -DataPath "C:\data\power.csv"

param(
    [string]$PoiKey = "",
    [string]$DataPath = "",
    [string]$ValueField = "",
    [string]$Unit = "",
    [string]$Multiplier = "",
    [string]$Interval = "",
    [switch]$NonInteractive
)

# Configuration
$BaseUrl = "https://msqisigttxosvnxfhfdn.supabase.co/storage/v1/object/public/releases"
$SupabaseUrl = "https://msqisigttxosvnxfhfdn.supabase.co"
$SupabaseAnonKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im1zcWlzaWd0dHhvc3ZueGZoZmRuIiwicm9sZSI6ImFub24iLCJpYXQiOjE3Njg4MTM2NDYsImV4cCI6MjA4NDM4OTY0Nn0.Idzca71FzW4SVlKlqHOsbh3JvMfzYH-jpCJP22rzSQ8"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) { Write-Output $args }
    $host.UI.RawUI.ForegroundColor = $fc
}

Clear-Host
Write-Host @"

    ___                    __  ____              ___
   /   | ____ ____  ____  / /_/ __ \__  _____  / (_)___ _
  / /| |/ __ `/ _ \/ __ \/ __/ / / / / / / _ \/ / / __ `/
 / ___ / /_/ /  __/ / / / /_/ /_/ / /_/ /  __/ / / /_/ /
/_/  |_\__, /\___/_/ /_/\__/\___\_\__,_/\___/_/_/\__,_/
      /____/

"@ -ForegroundColor Cyan

Write-Host "         Installation Agent de Collecte" -ForegroundColor Green
Write-Host ""

# Detect architecture
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-Host "  [OK] Systeme: Windows ($Arch)" -ForegroundColor Green

# Set paths
$InstallDir = "$env:ProgramFiles\Agentquelia"
$ConfigDir = "$env:APPDATA\Agentquelia"
$LogDir = "$env:APPDATA\Agentquelia\logs"

# Interactive mode
if (-not $NonInteractive) {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
    Write-Host "                    CONFIGURATION                        " -ForegroundColor Yellow
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
    Write-Host ""

    # 1. POI Key
    if ([string]::IsNullOrEmpty($PoiKey)) {
        Write-Host "  1. Cle POI *" -ForegroundColor Cyan
        Write-Host "     Format: sk_live_xxxxxxxxxxxxxxxx" -ForegroundColor Blue
        $PoiKey = Read-Host "     > "
        if ([string]::IsNullOrEmpty($PoiKey)) {
            Write-Host "  [ERREUR] Cle POI requise" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "  [OK] Cle POI: $($PoiKey.Substring(0, [Math]::Min(20, $PoiKey.Length)))..." -ForegroundColor Green
    }

    # 2. Data path
    if ([string]::IsNullOrEmpty($DataPath)) {
        Write-Host ""
        Write-Host "  2. Chemin du fichier de donnees *" -ForegroundColor Cyan
        Write-Host "     Exemples: C:\data\power.csv, D:\readings\data.json" -ForegroundColor Blue
        $DataPath = Read-Host "     > "
        if ([string]::IsNullOrEmpty($DataPath)) {
            Write-Host "  [ERREUR] Chemin requis" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "  [OK] Fichier: $DataPath" -ForegroundColor Green
    }

    # Detect file type
    $SourceType = "csv"
    $JsonPath = ""
    if ($DataPath -like "*.json") {
        $SourceType = "json"
    }

    # 3. Field/JSONPath
    Write-Host ""
    if ($SourceType -eq "json") {
        if ([string]::IsNullOrEmpty($ValueField)) {
            Write-Host "  3. JSONPath vers la valeur" -ForegroundColor Cyan
            Write-Host "     Exemples: `$.power, `$.data.reading, `$.meters[0].value" -ForegroundColor Blue
            $JsonPath = Read-Host "     > "
            if ([string]::IsNullOrEmpty($JsonPath)) { $JsonPath = "`$.power" }
        } else {
            $JsonPath = $ValueField
            Write-Host "  [OK] JSONPath: $JsonPath" -ForegroundColor Green
        }
    } else {
        if ([string]::IsNullOrEmpty($ValueField)) {
            Write-Host "  3. Nom de la colonne CSV" -ForegroundColor Cyan
            Write-Host "     Exemples: power_kw, value, reading" -ForegroundColor Blue
            $ValueField = Read-Host "     > "
            if ([string]::IsNullOrEmpty($ValueField)) { $ValueField = "power_kw" }
        } else {
            Write-Host "  [OK] Colonne: $ValueField" -ForegroundColor Green
        }
    }

    # 4. Unit
    Write-Host ""
    if ([string]::IsNullOrEmpty($Unit)) {
        Write-Host "  4. Unite de mesure finale" -ForegroundColor Cyan
        Write-Host "     Options: kW, MW, GW" -ForegroundColor Blue
        $Unit = Read-Host "     > "
        if ([string]::IsNullOrEmpty($Unit)) { $Unit = "kW" }
    } else {
        Write-Host "  [OK] Unite: $Unit" -ForegroundColor Green
    }

    # 5. Multiplier
    Write-Host ""
    if ([string]::IsNullOrEmpty($Multiplier)) {
        Write-Host "  5. Conversion de valeur (multiplicateur)" -ForegroundColor Cyan
        Write-Host "     Exemples:" -ForegroundColor Blue
        Write-Host "        * 1       = pas de conversion" -ForegroundColor Blue
        Write-Host "        * 0.001   = kW -> MW" -ForegroundColor Blue
        Write-Host "        * 1000    = MW -> kW" -ForegroundColor Blue
        $Multiplier = Read-Host "     [1] > "
        if ([string]::IsNullOrEmpty($Multiplier)) { $Multiplier = "1" }
    } else {
        Write-Host "  [OK] Multiplicateur: $Multiplier" -ForegroundColor Green
    }

    # 6. Interval
    Write-Host ""
    if ([string]::IsNullOrEmpty($Interval)) {
        Write-Host "  6. Intervalle de lecture (secondes)" -ForegroundColor Cyan
        Write-Host "     Frequence a laquelle l'agent lit et envoie les donnees" -ForegroundColor Blue
        $Interval = Read-Host "     [60] > "
        if ([string]::IsNullOrEmpty($Interval)) { $Interval = "60" }
    } else {
        Write-Host "  [OK] Intervalle: ${Interval}s" -ForegroundColor Green
    }
} else {
    # Non-interactive defaults
    $SourceType = "csv"
    $JsonPath = ""
    if ($DataPath -like "*.json") {
        $SourceType = "json"
        if ([string]::IsNullOrEmpty($ValueField)) { $JsonPath = "`$.power" } else { $JsonPath = $ValueField }
    }
    if ([string]::IsNullOrEmpty($ValueField)) { $ValueField = "power_kw" }
    if ([string]::IsNullOrEmpty($Unit)) { $Unit = "kW" }
    if ([string]::IsNullOrEmpty($Multiplier)) { $Multiplier = "1" }
    if ([string]::IsNullOrEmpty($Interval)) { $Interval = "60" }

    if ([string]::IsNullOrEmpty($PoiKey)) {
        Write-Host "  [ERREUR] Cle POI requise (-PoiKey)" -ForegroundColor Red
        exit 1
    }
    if ([string]::IsNullOrEmpty($DataPath)) {
        Write-Host "  [ERREUR] Chemin requis (-DataPath)" -ForegroundColor Red
        exit 1
    }
}

# Summary
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
Write-Host "                    RECAPITULATIF                        " -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
Write-Host ""
Write-Host "  [OK] Cle POI:      $($PoiKey.Substring(0, [Math]::Min(25, $PoiKey.Length)))..." -ForegroundColor Green
Write-Host "  [OK] Source:       $DataPath ($SourceType)" -ForegroundColor Green
if ($SourceType -eq "csv") {
    Write-Host "  [OK] Colonne:      $ValueField" -ForegroundColor Green
} else {
    Write-Host "  [OK] JSONPath:     $JsonPath" -ForegroundColor Green
}
Write-Host "  [OK] Unite:        $Unit" -ForegroundColor Green
if ($Multiplier -ne "1") {
    Write-Host "  [OK] Conversion:   x$Multiplier" -ForegroundColor Green
}
Write-Host "  [OK] Intervalle:   ${Interval}s" -ForegroundColor Green
Write-Host ""

if (-not $NonInteractive) {
    Write-Host "  Appuyez sur Entree pour continuer ou Ctrl+C pour annuler..." -ForegroundColor Cyan
    Read-Host
}

# Installation
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host "                    INSTALLATION                        " -ForegroundColor Blue
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host ""

# Download binary
Write-Host "  [>>] Telechargement de l'agent..."
$BinaryName = "agentquelia-windows-x86_64.exe"
$DownloadUrl = "$BaseUrl/$BinaryName"
$TempFile = "$env:TEMP\agentquelia.exe"

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
    Write-Host "  [OK] Agent telecharge" -ForegroundColor Green
} catch {
    Write-Host "  [ERREUR] Echec du telechargement" -ForegroundColor Red
    Write-Host "     URL: $DownloadUrl" -ForegroundColor Red
    exit 1
}

# Create directories
Write-Host "  [>>] Creation des repertoires..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

# Install binary
Write-Host "  [>>] Installation du binaire..."
Move-Item -Force $TempFile "$InstallDir\agentquelia.exe"
Write-Host "  [OK] Installe: $InstallDir\agentquelia.exe" -ForegroundColor Green

# Add to PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "  [>>] Ajout au PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "Machine")
    $env:Path = "$env:Path;$InstallDir"
}

# Create config
Write-Host "  [>>] Creation de la configuration..."
$ConfigFile = "$ConfigDir\agent.toml"

if ($SourceType -eq "csv") {
    $ConfigContent = @"
# Agentquelia Configuration
# Genere le $(Get-Date)

[agent]
instance_id = "$env:COMPUTERNAME"
polling_interval_secs = $Interval

[poi]
api_key = "$PoiKey"

[supabase]
url = "$SupabaseUrl"
anon_key = "$SupabaseAnonKey"

[source]
type = "csv"

[source.csv]
path = "$($DataPath -replace '\\', '\\')"
value_field = "$ValueField"
unit = "$Unit"
multiplier = $Multiplier
read_last_row = true

[logging]
level = "info"
directory = "$($LogDir -replace '\\', '\\')"
console_output = false
rotation = "daily"

[update]
enabled = false
"@
} else {
    $ConfigContent = @"
# Agentquelia Configuration
# Genere le $(Get-Date)

[agent]
instance_id = "$env:COMPUTERNAME"
polling_interval_secs = $Interval

[poi]
api_key = "$PoiKey"

[supabase]
url = "$SupabaseUrl"
anon_key = "$SupabaseAnonKey"

[source]
type = "json"

[source.json]
path = "$($DataPath -replace '\\', '\\')"
json_path = "$JsonPath"
unit = "$Unit"
multiplier = $Multiplier

[logging]
level = "info"
directory = "$($LogDir -replace '\\', '\\')"
console_output = false
rotation = "daily"

[update]
enabled = false
"@
}

$ConfigContent | Out-File -FilePath $ConfigFile -Encoding utf8
Write-Host "  [OK] Configuration: $ConfigFile" -ForegroundColor Green

# Install as Windows Service
Write-Host "  [>>] Installation du service Windows..."
try {
    & "$InstallDir\agentquelia.exe" install 2>$null
    Write-Host "  [OK] Service installe" -ForegroundColor Green
} catch {
    Write-Host "  [!] Service non installe (executez en admin)" -ForegroundColor Yellow
}

# Summary
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "            [OK] INSTALLATION TERMINEE !                 " -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host ""
Write-Host "  POI:         $($PoiKey.Substring(0, [Math]::Min(25, $PoiKey.Length)))..."
Write-Host "  Source:      $DataPath"
if ($SourceType -eq "csv") {
    Write-Host "  Colonne:     $ValueField"
} else {
    Write-Host "  JSONPath:    $JsonPath"
}
Write-Host "  Unite:       $Unit"
if ($Multiplier -ne "1") {
    Write-Host "  Conversion:  x$Multiplier"
}
Write-Host "  Intervalle:  ${Interval}s"
Write-Host "  Config:      $ConfigFile"
Write-Host "  Logs:        $LogDir"
Write-Host ""
Write-Host "Commandes utiles:" -ForegroundColor Cyan
Write-Host "  sc query agentquelia              # Statut du service"
Write-Host "  sc start agentquelia              # Demarrer"
Write-Host "  sc stop agentquelia               # Arreter"
Write-Host "  agentquelia run                   # Lancer manuellement"
Write-Host ""
