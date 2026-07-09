#!/usr/bin/env bash
set -euo pipefail

pkill -f "python3.*proton.*nhllegacy" 2>/dev/null || true
pkill -f "steam.exe.*nhllegacy" 2>/dev/null || true
pkill -f "nhllegacy.exe" 2>/dev/null || true

echo "NHL Legacy processes terminated."
