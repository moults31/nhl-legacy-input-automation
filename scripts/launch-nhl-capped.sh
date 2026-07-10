#!/usr/bin/env bash
set -euo pipefail

mangohud --mangohud-config "fps_limit=10,no_display" \
  bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh "$@"
