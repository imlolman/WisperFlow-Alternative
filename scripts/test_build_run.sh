#!/usr/bin/env bash
# Test from source (no build), then build once, then kill existing app → replace in /Applications → launch.
# No loops. Run this when you want to try the built app.
set -euo pipefail

cd "$(dirname "$0")/.."
APP_NAME="WisperFlow Alternative"

# 1) Quick test from source (imports only; no build, no window)
echo "==> Quick test (source)..."
python -c "
from wisperflow.ui import run_settings
from wisperflow.__main__ import main
print('  imports OK')
" 2>/dev/null || { echo "  Source test failed (fix code first)"; exit 1; }

# 2) Build once
echo "==> Build..."
./scripts/build_macos.sh

# 3) Kill existing, replace, launch
echo "==> Replacing app and launching..."
pkill -f "$APP_NAME" 2>/dev/null || true
sleep 1
cp -R "dist/${APP_NAME}.app" /Applications/
open -a "$APP_NAME"

echo ""
echo "==> Done. App is running from /Applications."
