#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing thumbdesk. for Linux (AppImage)..."
API_URL="https://api.github.com/repos/orionopenwow/ThumbDesk/releases/latest"
APPIMAGE_URL="$(curl -fsSL "$API_URL" | grep -oE 'https://[^"]+\.AppImage' | head -n 1 || true)"

if [ -z "$APPIMAGE_URL" ]; then
  echo "No .AppImage asset found on the latest release yet."
  echo "Visit: https://github.com/orionopenwow/ThumbDesk/releases/latest"
  exit 1
fi

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
DEST="$INSTALL_DIR/thumbdesk.AppImage"
curl -fSL "$APPIMAGE_URL" -o "$DEST"
chmod +x "$DEST"
echo "==> Installed to $DEST"
echo "Run with: $DEST"
