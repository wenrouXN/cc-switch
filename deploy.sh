#!/usr/bin/env bash
set -euo pipefail

# Deploy CC Switch Web UI as a systemd service
# Usage: sudo bash deploy.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVICE_NAME="cc-switch-web"
INSTALL_DIR="/vol1/1000/config/share/ccswitch-web-official"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

# 1. Copy binary (already includes embedded web assets)
echo "Deploying cc-switch binary to ${INSTALL_DIR}..."
cp "${SCRIPT_DIR}/target/release/cc-switch" "${INSTALL_DIR}/cc-switch"
chmod +x "${INSTALL_DIR}/cc-switch"

# 2. Install systemd service
echo "Installing systemd service..."
cp "${SCRIPT_DIR}/${SERVICE_NAME}.service" "${SERVICE_FILE}"
systemctl daemon-reload

# 3. Enable and start
echo "Enabling and starting service..."
systemctl enable "${SERVICE_NAME}"
systemctl restart "${SERVICE_NAME}"

echo ""
echo "Done! CC Switch Web UI is running on port 17667"
echo "  Status:  systemctl status ${SERVICE_NAME}"
echo "  Logs:    journalctl -u ${SERVICE_NAME} -f"
echo "  Stop:    systemctl stop ${SERVICE_NAME}"
echo "  Disable: systemctl disable ${SERVICE_NAME}"
