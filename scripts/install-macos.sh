#!/bin/bash
set -e

# Agentquelia macOS Installation Script
# =====================================

INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/Library/Application Support/agentquelia"
LOG_DIR="$HOME/Library/Logs/agentquelia"
PLIST_DIR="$HOME/Library/LaunchAgents"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Agentquelia macOS Installation${NC}"
echo "================================"
echo

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    BINARY_NAME="agentquelia-macos-aarch64"
elif [ "$ARCH" = "x86_64" ]; then
    BINARY_NAME="agentquelia-macos-x86_64"
else
    echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
    exit 1
fi

# Check if running from source directory
if [ -f "./target/release/agentquelia" ]; then
    BINARY_PATH="./target/release/agentquelia"
    echo "Installing from local build..."
elif [ -n "$BINARY_URL" ]; then
    # Download from URL
    echo "Downloading $BINARY_NAME..."
    BINARY_PATH="/tmp/agentquelia"
    curl -L "$BINARY_URL" -o "$BINARY_PATH"
    chmod +x "$BINARY_PATH"
else
    echo -e "${RED}Error: No binary found. Either build locally or set BINARY_URL${NC}"
    echo "To build locally: cargo build --release"
    exit 1
fi

# Create directories
echo "Creating directories..."
mkdir -p "$CONFIG_DIR"
mkdir -p "$LOG_DIR"
mkdir -p "$PLIST_DIR"

# Install binary
echo "Installing binary to $INSTALL_DIR..."
if [ -w "$INSTALL_DIR" ]; then
    cp "$BINARY_PATH" "$INSTALL_DIR/agentquelia"
else
    sudo cp "$BINARY_PATH" "$INSTALL_DIR/agentquelia"
fi
chmod +x "$INSTALL_DIR/agentquelia"

# Copy example config if no config exists
CONFIG_FILE="$CONFIG_DIR/agent.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating example configuration..."
    if [ -f "./config/agent.example.toml" ]; then
        cp "./config/agent.example.toml" "$CONFIG_FILE"
    else
        cat > "$CONFIG_FILE" << 'CONFIGEOF'
[agent]
instance_id = "poi-001"
polling_interval_secs = 60

[poi]
api_key = "${AGENTQUELIA_POI_KEY}"

[supabase]
url = "https://msqisigttxosvnxfhfdn.supabase.co"
anon_key = "${SUPABASE_ANON_KEY}"

[source]
type = "csv"

[source.csv]
path = "/path/to/your/data.csv"
value_field = "power_kw"
unit = "kW"

[logging]
level = "info"
CONFIGEOF
    fi
    echo -e "${YELLOW}IMPORTANT: Edit $CONFIG_FILE with your settings${NC}"
fi

# Install launchd service
echo "Installing launchd service..."
"$INSTALL_DIR/agentquelia" install --user 2>/dev/null || true

echo
echo -e "${GREEN}Installation complete!${NC}"
echo
echo "Next steps:"
echo "1. Edit your configuration: $CONFIG_FILE"
echo "2. Set environment variables:"
echo "   export AGENTQUELIA_POI_KEY='your_poi_key'"
echo "   export SUPABASE_ANON_KEY='your_supabase_key'"
echo "3. Test the agent: agentquelia run --config \"$CONFIG_FILE\""
echo "4. Start the service: launchctl load ~/Library/LaunchAgents/com.agentquelia.agent.plist"
echo
echo "Logs: $LOG_DIR"
