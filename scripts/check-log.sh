#!/usr/bin/env bash
# Quantify drift between screenshots captured and log entries written.
# Exits 0 if the gap is within tolerance, non-zero otherwise.
#
# Usage: ./scripts/check-log.sh <run_id> [max_gap]
#   run_id    - subdirectory name under screenshots/
#   max_gap   - allowed difference between screenshot count and log count (default: 5)

set -euo pipefail

RUN_ID="${1:?Usage: check-log.sh <run_id> [max_gap]}"
MAX_GAP="${2:-5}"
DIR="screenshots/${RUN_ID}"

if [ ! -d "$DIR" ]; then
    echo "CHECK-LOG: directory not found: $DIR"
    exit 1
fi

PNG_COUNT=$(find "$DIR" -maxdepth 1 -name '*.png' | wc -l)
LOG_COUNT=$(wc -l < "$DIR/run_log.jsonl" 2>/dev/null || echo 0)
GAP=$((PNG_COUNT - LOG_COUNT))

if [ "$GAP" -gt "$MAX_GAP" ]; then
    echo "CHECK-LOG: DRIFT DETECTED"
    echo "  screenshots: $PNG_COUNT"
    echo "  log entries: $LOG_COUNT"
    echo "  gap:         $GAP (max allowed: $MAX_GAP)"
    echo "  run_dir:     $DIR"
    exit 1
fi

echo "CHECK-LOG: OK (screenshots=$PNG_COUNT, log_entries=$LOG_COUNT, gap=$GAP, max=$MAX_GAP)"
exit 0
