#!/usr/bin/env bash
set -euo pipefail

while IFS= read -r pid; do
    kill "$pid" 2>/dev/null || true
done < <(pgrep -f "nhllegacy.exe|steam.exe.*nhllegacy|python3.*proton.*nhllegacy" 2>/dev/null || true)

echo "NHL Legacy processes terminated."
