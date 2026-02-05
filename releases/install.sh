#!/bin/bash
set -e

# ============================================
# Agentquelia Installer - Installation Interactive
# ============================================
# Usage: curl -sSL "https://URL/install.sh" | bash
# Or:    curl -sSL "https://URL/install.sh" | bash -s -- --key sk_live_xxx --path /data/file.csv

# Parse arguments
POI_KEY="${POI_KEY:-}"
DATA_PATH="${DATA_PATH:-}"
UNIT="${UNIT:-}"
VALUE_FIELD="${VALUE_FIELD:-}"
POLLING_INTERVAL="${POLLING_INTERVAL:-}"
MULTIPLIER="${MULTIPLIER:-}"
NONINTERACTIVE="${NONINTERACTIVE:-}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --key) POI_KEY="$2"; shift 2 ;;
        --path) DATA_PATH="$2"; shift 2 ;;
        --unit) UNIT="$2"; shift 2 ;;
        --field) VALUE_FIELD="$2"; shift 2 ;;
        --interval) POLLING_INTERVAL="$2"; shift 2 ;;
        --multiplier) MULTIPLIER="$2"; shift 2 ;;
        --yes|-y) NONINTERACTIVE="true"; shift ;;
        *) shift ;;
    esac
done

# Configuration Supabase
BASE_URL="https://msqisigttxosvnxfhfdn.supabase.co/storage/v1/object/public/releases"
SUPABASE_URL="https://msqisigttxosvnxfhfdn.supabase.co"
SUPABASE_ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im1zcWlzaWd0dHhvc3ZueGZoZmRuIiwicm9sZSI6ImFub24iLCJpYXQiOjE3Njg4MTM2NDYsImV4cCI6MjA4NDM4OTY0Nn0.Idzca71FzW4SVlKlqHOsbh3JvMfzYH-jpCJP22rzSQ8"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

clear
echo -e "${CYAN}"
cat << "EOF"
    ___                    __  ____              ___
   /   | ____ ____  ____  / /_/ __ \__  _____  / (_)___ _
  / /| |/ __ `/ _ \/ __ \/ __/ / / / / / / _ \/ / / __ `/
 / ___ / /_/ /  __/ / / / /_/ /_/ / /_/ /  __/ / / /_/ /
/_/  |_\__, /\___/_/ /_/\__/\___\_\__,_/\___/_/_/\__,_/
      /____/
EOF
echo -e "${NC}"
echo -e "${GREEN}         Installation Agent de Collecte${NC}"
echo

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)  PLATFORM="linux" ;;
    Darwin*) PLATFORM="macos" ;;
    *)       echo -e "${RED}❌ OS non supporté: $OS${NC}"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH="x86_64" ;;
    aarch64|arm64)  ARCH="aarch64" ;;
    *)              echo -e "${RED}❌ Architecture non supportée: $ARCH${NC}"; exit 1 ;;
esac

echo -e "  ${GREEN}✓${NC} Système: $PLATFORM ($ARCH)"

# Set paths
if [ "$PLATFORM" = "macos" ]; then
    INSTALL_DIR="/usr/local/bin"
    CONFIG_DIR="$HOME/Library/Application Support/agentquelia"
    LOG_DIR="$HOME/Library/Logs/agentquelia"
else
    INSTALL_DIR="/usr/local/bin"
    CONFIG_DIR="/etc/agentquelia"
    LOG_DIR="/var/log/agentquelia"
fi

# Interactive mode
if [ -z "$NONINTERACTIVE" ]; then
    echo
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}                    CONFIGURATION                        ${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo

    # 1. POI Key
    if [ -z "$POI_KEY" ]; then
        echo -e "  ${CYAN}1.${NC} Clé POI ${RED}*${NC}"
        echo -e "     ${BLUE}Format: sk_live_xxxxxxxxxxxxxxxx${NC}"
        echo -n "     > "
        read -r POI_KEY < /dev/tty
        if [ -z "$POI_KEY" ]; then
            echo -e "${RED}❌ Clé POI requise${NC}"
            exit 1
        fi
    else
        echo -e "  ${GREEN}✓${NC} Clé POI: ${POI_KEY:0:20}..."
    fi

    # 2. Data path
    if [ -z "$DATA_PATH" ]; then
        echo
        echo -e "  ${CYAN}2.${NC} Chemin du fichier de données ${RED}*${NC}"
        echo -e "     ${BLUE}Exemples: /var/data/power.csv, /home/user/readings.json${NC}"
        echo -n "     > "
        read -r DATA_PATH < /dev/tty
        if [ -z "$DATA_PATH" ]; then
            echo -e "${RED}❌ Chemin requis${NC}"
            exit 1
        fi
    else
        echo -e "  ${GREEN}✓${NC} Fichier: $DATA_PATH"
    fi

    # Detect file type
    SOURCE_TYPE="csv"
    JSON_PATH=""
    if [[ "$DATA_PATH" == *.json ]]; then
        SOURCE_TYPE="json"
    fi

    # 3. Field/JSONPath
    echo
    if [ "$SOURCE_TYPE" = "json" ]; then
        if [ -z "$VALUE_FIELD" ]; then
            echo -e "  ${CYAN}3.${NC} JSONPath vers la valeur"
            echo -e "     ${BLUE}Exemples: \$.power, \$.data.reading, \$.meters[0].value${NC}"
            echo -n "     > "
            read -r JSON_PATH < /dev/tty
            JSON_PATH="${JSON_PATH:-\$.power}"
        else
            JSON_PATH="$VALUE_FIELD"
            echo -e "  ${GREEN}✓${NC} JSONPath: $JSON_PATH"
        fi
    else
        if [ -z "$VALUE_FIELD" ]; then
            echo -e "  ${CYAN}3.${NC} Nom de la colonne CSV"
            echo -e "     ${BLUE}Exemples: power_kw, value, reading${NC}"
            echo -n "     > "
            read -r VALUE_FIELD < /dev/tty
            VALUE_FIELD="${VALUE_FIELD:-power_kw}"
        else
            echo -e "  ${GREEN}✓${NC} Colonne: $VALUE_FIELD"
        fi
    fi

    # 4. Unit selection
    echo
    if [ -z "$UNIT" ]; then
        echo -e "  ${CYAN}4.${NC} Unité de mesure finale (celle affichée sur la carte)"
        echo -e "     ${BLUE}Options: kW, MW, GW${NC}"
        echo -n "     > "
        read -r UNIT < /dev/tty
        UNIT="${UNIT:-kW}"
    else
        echo -e "  ${GREEN}✓${NC} Unité: $UNIT"
    fi

    # 5. Multiplier (conversion)
    echo
    if [ -z "$MULTIPLIER" ]; then
        echo -e "  ${CYAN}5.${NC} Conversion de valeur (multiplicateur)"
        echo -e "     ${BLUE}Exemples:${NC}"
        echo -e "        ${BLUE}• 1       = pas de conversion${NC}"
        echo -e "        ${BLUE}• 0.001   = kW → MW${NC}"
        echo -e "        ${BLUE}• 1000    = MW → kW${NC}"
        echo -e "        ${BLUE}• 0.000001 = kW → GW${NC}"
        echo -n "     [1] > "
        read -r MULTIPLIER < /dev/tty
        MULTIPLIER="${MULTIPLIER:-1}"
    else
        echo -e "  ${GREEN}✓${NC} Multiplicateur: $MULTIPLIER"
    fi

    # 6. Polling interval
    echo
    if [ -z "$POLLING_INTERVAL" ]; then
        echo -e "  ${CYAN}6.${NC} Intervalle de lecture (secondes)"
        echo -e "     ${BLUE}Fréquence à laquelle l'agent lit et envoie les données${NC}"
        echo -n "     [60] > "
        read -r POLLING_INTERVAL < /dev/tty
        POLLING_INTERVAL="${POLLING_INTERVAL:-60}"
    else
        echo -e "  ${GREEN}✓${NC} Intervalle: ${POLLING_INTERVAL}s"
    fi

else
    # Non-interactive defaults
    SOURCE_TYPE="csv"
    JSON_PATH=""
    if [[ "$DATA_PATH" == *.json ]]; then
        SOURCE_TYPE="json"
        JSON_PATH="${VALUE_FIELD:-\$.power}"
    fi
    VALUE_FIELD="${VALUE_FIELD:-power_kw}"
    UNIT="${UNIT:-kW}"
    MULTIPLIER="${MULTIPLIER:-1}"
    POLLING_INTERVAL="${POLLING_INTERVAL:-60}"

    if [ -z "$POI_KEY" ]; then
        echo -e "${RED}❌ Clé POI requise (--key)${NC}"
        exit 1
    fi
    if [ -z "$DATA_PATH" ]; then
        echo -e "${RED}❌ Chemin de données requis (--path)${NC}"
        exit 1
    fi
fi

# Confirmation summary
echo
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}                    RÉCAPITULATIF                        ${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "  ${GREEN}✓${NC} Clé POI:      ${POI_KEY:0:25}..."
echo -e "  ${GREEN}✓${NC} Source:       $DATA_PATH ($SOURCE_TYPE)"
if [ "$SOURCE_TYPE" = "csv" ]; then
    echo -e "  ${GREEN}✓${NC} Colonne:      $VALUE_FIELD"
else
    echo -e "  ${GREEN}✓${NC} JSONPath:     $JSON_PATH"
fi
echo -e "  ${GREEN}✓${NC} Unité:        $UNIT"
if [ "$MULTIPLIER" != "1" ]; then
    echo -e "  ${GREEN}✓${NC} Conversion:   ×$MULTIPLIER"
fi
echo -e "  ${GREEN}✓${NC} Intervalle:   ${POLLING_INTERVAL}s"
echo

if [ -z "$NONINTERACTIVE" ]; then
    echo -e "  ${CYAN}Appuyez sur Entrée pour continuer ou Ctrl+C pour annuler...${NC}"
    read -r < /dev/tty
fi

# Download binary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}                    INSTALLATION                        ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "  📥 Téléchargement de l'agent..."
BINARY_NAME="agentquelia-${PLATFORM}-${ARCH}"
DOWNLOAD_URL="${BASE_URL}/${BINARY_NAME}"

if command -v curl &> /dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o /tmp/agentquelia 2>/dev/null || {
        echo -e "  ${RED}❌ Échec du téléchargement${NC}"
        echo -e "     URL: $DOWNLOAD_URL"
        exit 1
    }
else
    wget -q "$DOWNLOAD_URL" -O /tmp/agentquelia 2>/dev/null || {
        echo -e "  ${RED}❌ Échec du téléchargement${NC}"
        exit 1
    }
fi

chmod +x /tmp/agentquelia
echo -e "  ${GREEN}✓${NC} Agent téléchargé"

# Install binary
echo -e "  📦 Installation du binaire..."
if [ -w "$INSTALL_DIR" ]; then
    mv /tmp/agentquelia "$INSTALL_DIR/agentquelia"
else
    sudo mv /tmp/agentquelia "$INSTALL_DIR/agentquelia"
fi
echo -e "  ${GREEN}✓${NC} Installé: $INSTALL_DIR/agentquelia"

# Create directories
mkdir -p "$CONFIG_DIR" 2>/dev/null || sudo mkdir -p "$CONFIG_DIR"
mkdir -p "$LOG_DIR" 2>/dev/null || sudo mkdir -p "$LOG_DIR"

# Create config file
echo -e "  ⚙️  Création de la configuration..."
CONFIG_FILE="$CONFIG_DIR/agent.toml"

if [ "$SOURCE_TYPE" = "csv" ]; then
    CONFIG_CONTENT="# Agentquelia Configuration
# Généré le $(date)

[agent]
instance_id = \"$(hostname)\"
polling_interval_secs = $POLLING_INTERVAL

[poi]
api_key = \"$POI_KEY\"

[supabase]
url = \"$SUPABASE_URL\"
anon_key = \"$SUPABASE_ANON_KEY\"

[source]
type = \"csv\"

[source.csv]
path = \"$DATA_PATH\"
value_field = \"$VALUE_FIELD\"
unit = \"$UNIT\"
multiplier = $MULTIPLIER
read_last_row = true

[logging]
level = \"info\"
console_output = false
rotation = \"daily\"

[update]
enabled = false
"
else
    CONFIG_CONTENT="# Agentquelia Configuration
# Généré le $(date)

[agent]
instance_id = \"$(hostname)\"
polling_interval_secs = $POLLING_INTERVAL

[poi]
api_key = \"$POI_KEY\"

[supabase]
url = \"$SUPABASE_URL\"
anon_key = \"$SUPABASE_ANON_KEY\"

[source]
type = \"json\"

[source.json]
path = \"$DATA_PATH\"
json_path = \"$JSON_PATH\"
unit = \"$UNIT\"
multiplier = $MULTIPLIER

[logging]
level = \"info\"
console_output = false
rotation = \"daily\"

[update]
enabled = false
"
fi

if [ -w "$CONFIG_DIR" ]; then
    echo "$CONFIG_CONTENT" > "$CONFIG_FILE"
else
    echo "$CONFIG_CONTENT" | sudo tee "$CONFIG_FILE" > /dev/null
fi
echo -e "  ${GREEN}✓${NC} Configuration: $CONFIG_FILE"

# Install as service
echo -e "  🔧 Installation du service..."
if [ "$PLATFORM" = "linux" ]; then
    sudo "$INSTALL_DIR/agentquelia" install 2>/dev/null || true
    sudo systemctl daemon-reload 2>/dev/null || true
    sudo systemctl enable agentquelia 2>/dev/null || true
    sudo systemctl start agentquelia 2>/dev/null || true
elif [ "$PLATFORM" = "macos" ]; then
    "$INSTALL_DIR/agentquelia" install --user 2>/dev/null || true
    launchctl load ~/Library/LaunchAgents/com.agentquelia.agent.plist 2>/dev/null || true
fi
echo -e "  ${GREEN}✓${NC} Service installé"

# Summary
echo
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}            ✅ INSTALLATION TERMINÉE !                   ${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "  📍 POI:         ${POI_KEY:0:25}..."
echo -e "  📁 Source:      $DATA_PATH"
if [ "$SOURCE_TYPE" = "csv" ]; then
    echo -e "  📊 Colonne:     $VALUE_FIELD"
else
    echo -e "  📊 JSONPath:    $JSON_PATH"
fi
echo -e "  📐 Unité:       $UNIT"
if [ "$MULTIPLIER" != "1" ]; then
    echo -e "  🔄 Conversion:  ×$MULTIPLIER"
fi
echo -e "  ⏱️  Intervalle:  ${POLLING_INTERVAL}s"
echo -e "  📝 Config:      $CONFIG_FILE"
echo -e "  📋 Logs:        $LOG_DIR"
echo

if [ "$PLATFORM" = "linux" ]; then
    echo -e "${CYAN}Commandes utiles:${NC}"
    echo "  sudo systemctl status agentquelia    # Statut"
    echo "  sudo systemctl restart agentquelia   # Redémarrer"
    echo "  sudo journalctl -u agentquelia -f    # Logs"
else
    echo -e "${CYAN}Commandes utiles:${NC}"
    echo "  agentquelia status                   # Statut"
    echo "  tail -f ~/Library/Logs/agentquelia/*.log  # Logs"
fi
echo
