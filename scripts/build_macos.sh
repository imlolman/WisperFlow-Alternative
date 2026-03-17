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
    --hidden-import wisperflow.onboarding \
    --collect-data whisper \
    --collect-data torch \
    --osx-bundle-identifier com.wisperflow.app \
    wisperflow/__main__.py

# Patch Info.plist
PLIST="dist/${APP_NAME}.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :LSUIElement bool true" "$PLIST" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :LSUIElement true" "$PLIST"
/usr/libexec/PlistBuddy -c "Add :NSMicrophoneUsageDescription string 'WisperFlow Alternative needs microphone access to transcribe speech.'" "$PLIST" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :NSMicrophoneUsageDescription 'WisperFlow Alternative needs microphone access to transcribe speech.'" "$PLIST"

# Create entitlements file for mic access
ENTITLEMENTS="dist/entitlements.plist"
cat > "$ENTITLEMENTS" <<ENTEOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.device.audio-input</key>
    <true/>
</dict>
</plist>
ENTEOF

# Re-sign the app so the patched Info.plist is sealed into the signature
echo "==> Re-signing app bundle..."
codesign --force --deep --sign - --entitlements "$ENTITLEMENTS" "dist/${APP_NAME}.app"
rm -f "$ENTITLEMENTS"

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
