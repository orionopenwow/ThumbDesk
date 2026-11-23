#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing thumbdesk. for macOS..."
API_URL="https://api.github.com/repos/orionopenwow/ThumbDesk/releases/latest"
DMG_URL="$(curl -fsSL "$API_URL" | grep -oE 'https://[^"]+\.dmg' | head -n 1 || true)"

if [ -z "$DMG_URL" ]; then
  echo "No .dmg asset found on the latest release yet."
  echo "Opening releases page: https://github.com/orionopenwow/ThumbDesk/releases/latest"
  open "https://github.com/orionopenwow/ThumbDesk/releases/latest" || true
  exit 1
fi

TMP_DMG="/tmp/ThumbDesk-latest.dmg"
curl -fSL "$DMG_URL" -o "$TMP_DMG"
echo "==> Mounting $TMP_DMG..."
hdiutil attach "$TMP_DMG" -nobrowse
echo "==> Drag ThumbDesk.app into /Applications to finish installing."
