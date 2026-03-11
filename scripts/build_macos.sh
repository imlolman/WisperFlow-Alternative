#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

APP_NAME="WisperFlow Alternative"
DMG_NAME="WisperFlow-Alternative-Installer"

echo "==> Installing build deps..."
pip install -q pyinstaller

echo "==> Building ${APP_NAME}.app..."
pyinstaller \
    --name "$APP_NAME" \
    --windowed \
    --noconfirm \
    --add-data "wisperflow/assets:wf_assets" \
    --hidden-import rumps \
    --hidden-import pynput \
    --hidden-import pynput.keyboard \
    --hidden-import pynput.keyboard._darwin \
    --hidden-import pynput.mouse \
    --hidden-import pynput.mouse._darwin \
    --hidden-import whisper \
    --hidden-import sounddevice \
    --hidden-import webview \
    --hidden-import torch \
    --hidden-import numpy \
    --hidden-import wisperflow \
    --hidden-import wisperflow.app \
    --hidden-import wisperflow.ipc \
    --hidden-import wisperflow.config \
    --hidden-import wisperflow.clipboard \
    --hidden-import wisperflow.overlay \
    --hidden-import wisperflow.shortcuts \
    --hidden-import wisperflow.transcriber \
    --hidden-import wisperflow.ui \
    --collect-data whisper \
    --collect-data torch \
    --osx-bundle-identifier com.wisperflow.app \
    wisperflow/__main__.py

# Patch Info.plist: agent app (no dock icon)
PLIST="dist/${APP_NAME}.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :LSUIElement bool true" "$PLIST" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :LSUIElement true" "$PLIST"

echo "==> Creating DMG installer..."

# Clean previous
rm -rf "dist/dmg" "dist/${DMG_NAME}.dmg"
mkdir -p "dist/dmg"

# Copy app into staging dir and add Applications symlink
cp -R "dist/${APP_NAME}.app" "dist/dmg/"
ln -s /Applications "dist/dmg/Applications"

# Create DMG
hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "dist/dmg" \
    -ov \
    -format UDZO \
    "dist/${DMG_NAME}.dmg"

rm -rf "dist/dmg"

echo ""
echo "==> Done!"
echo "    dist/${DMG_NAME}.dmg"
echo ""
echo "    Double-click the DMG, drag the app to Applications."
