#!/bin/bash
set -e

# ============================================
# Agentquelia Universal Installation Script
# ============================================
# Usage: curl -sSL https://your-url/install.sh | bash
# Or with POI key: curl -sSL https://your-url/install.sh | POI_KEY=xxx bash

# Configuration - CHANGE THESE URLs
BASE_URL="${BASE_URL:-https://msqisigttxosvnxfhfdn.supabase.co/storage/v1/object/public/releases}"
SUPABASE_URL="https://msqisigttxosvnxfhfdn.supabase.co"
SUPABASE_ANON_KEY="${SUPABASE_ANON_KEY:-}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     Agentquelia Installation Script    ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        PLATFORM="linux"
        INSTALL_DIR="/usr/local/bin"
        CONFIG_DIR="/etc/agentquelia"
        ;;
    Darwin*)
        PLATFORM="macos"
        INSTALL_DIR="/usr/local/bin"
        CONFIG_DIR="$HOME/Library/Application Support/agentquelia"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows"
        INSTALL_DIR="$PROGRAMFILES/agentquelia"
        CONFIG_DIR="$APPDATA/agentquelia"
        ;;
    *)
        echo -e "${RED}Error: Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH="x86_64" ;;
    aarch64|arm64)  ARCH="aarch64" ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

BINARY_NAME="agentquelia-${PLATFORM}-${ARCH}"
if [ "$PLATFORM" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

echo "Detected: $PLATFORM ($ARCH)"
echo "Binary: $BINARY_NAME"
echo

# Download binary
echo "Downloading agentquelia..."
DOWNLOAD_URL="${BASE_URL}/${BINARY_NAME}"

if command -v curl &> /dev/null; then
    curl -fSL "$DOWNLOAD_URL" -o /tmp/agentquelia
elif command -v wget &> /dev/null; then
    wget -q "$DOWNLOAD_URL" -O /tmp/agentquelia
else
    echo -e "${RED}Error: curl or wget required${NC}"
    exit 1
fi

chmod +x /tmp/agentquelia

# Install binary
echo "Installing to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    mv /tmp/agentquelia "$INSTALL_DIR/agentquelia"
else
    sudo mv /tmp/agentquelia "$INSTALL_DIR/agentquelia"
fi

# Create config directory
echo "Creating config directory..."
mkdir -p "$CONFIG_DIR" 2>/dev/null || sudo mkdir -p "$CONFIG_DIR"

# Create config if POI_KEY is provided
CONFIG_FILE="$CONFIG_DIR/agent.toml"
if [ -n "$POI_KEY" ] && [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating configuration..."
    cat > /tmp/agent.toml << EOF
[agent]
instance_id = "$(hostname)"
polling_interval_secs = 60

[poi]
api_key = "$POI_KEY"

[supabase]
url = "$SUPABASE_URL"
anon_key = "$SUPABASE_ANON_KEY"

[source]
type = "csv"

[source.csv]
path = "/path/to/your/data.csv"
value_field = "power_kw"
unit = "kW"

[logging]
level = "info"
EOF

    if [ -w "$CONFIG_DIR" ]; then
        mv /tmp/agent.toml "$CONFIG_FILE"
    else
        sudo mv /tmp/agent.toml "$CONFIG_FILE"
    fi

    echo -e "${YELLOW}Config created at: $CONFIG_FILE${NC}"
    echo -e "${YELLOW}Please edit source.csv.path with your data file path${NC}"
fi

# Install as service
echo "Installing as service..."
if [ "$PLATFORM" = "linux" ]; then
    sudo "$INSTALL_DIR/agentquelia" install 2>/dev/null || true
    echo -e "${GREEN}Start with: sudo systemctl start agentquelia${NC}"
elif [ "$PLATFORM" = "macos" ]; then
    "$INSTALL_DIR/agentquelia" install --user 2>/dev/null || true
    echo -e "${GREEN}Start with: launchctl load ~/Library/LaunchAgents/com.agentquelia.agent.plist${NC}"
fi

echo
echo -e "${GREEN}✓ Installation complete!${NC}"
echo
echo "Next steps:"
echo "1. Edit config: $CONFIG_FILE"
echo "2. Set your CSV/JSON/API source path"
echo "3. Start the service"
echo
echo "Test manually: agentquelia -c \"$CONFIG_FILE\" run"
