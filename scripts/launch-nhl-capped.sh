#!/usr/bin/env bash
set -euo pipefail

MANGOHUD_CONFIG="fps_limit=10,no_display" mangohud \
  bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh "$@"
