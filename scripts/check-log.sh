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

# Plan-vs-script correspondence check:
# For each log entry with decision=navigate, check if the plan mentions
# a repeat-count pattern (Nx, ×N, N times) and flag if the subsequent
# daemon command used individual taps instead of scroll().
WARNINGS=0
if [ -f "$DIR/daemon_events.jsonl" ] && [ -f "$DIR/run_log.jsonl" ]; then
    while IFS= read -r line; do
        step=$(echo "$line" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('step',''))" 2>/dev/null || echo "")
        plan=$(echo "$line" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('plan',''))" 2>/dev/null || echo "")
        decision=$(echo "$line" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('decision',''))" 2>/dev/null || echo "")
        if [ "$decision" != "navigate" ] || [ -z "$plan" ]; then
            continue
        fi
        # Check if plan mentions a count pattern (e.g., "3x dpad_down", "↓×6", "3 times")
        if echo "$plan" | grep -qiP '(\d+\s*x\s*(dpad_|↓)|[↓↑]×\d+|\d+\s*times|\d+\s*presses)' > /dev/null 2>&1; then
            # Plan has a count — check the subsequent daemon command for scroll()
            # Find the daemon command that follows this log step's screenshot
            script_found=0
            while IFS= read -r dline; do
                devent=$(echo "$dline" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('event',''))" 2>/dev/null || echo "")
                if [ "$devent" = "command" ]; then
                    dscript=$(echo "$dline" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('script',''))" 2>/dev/null || echo "")
                    if echo "$dscript" | grep -q 'scroll(' > /dev/null 2>&1; then
                        script_found=1
                        break
                    elif echo "$dscript" | grep -q 'tap(' > /dev/null 2>&1; then
                        # Got a tap before a scroll — flag this
                        if [ $script_found -eq 0 ]; then
                            script_found=2
                            break
                        fi
                    fi
                fi
            done < <(grep -A1 "step_$(printf '%03d' "$step")" "$DIR/daemon_events.jsonl" 2>/dev/null || true)

            if [ "$script_found" = "2" ]; then
                echo "CHECK-LOG: WARNING — step $step plan mentions count but used individual tap()"
                echo "  plan: $plan"
                WARNINGS=$((WARNINGS + 1))
            fi
        fi
    done < "$DIR/run_log.jsonl"
fi

if [ "$WARNINGS" -gt 0 ]; then
    echo "CHECK-LOG: WARNING — $WARNINGS plan-vs-script mismatch(es) found"
    echo "  run_dir: $DIR"
    # Non-fatal: warnings only, exit 0
fi

echo "CHECK-LOG: OK (screenshots=$PNG_COUNT, log_entries=$LOG_COUNT, gap=$GAP, max=$MAX_GAP)"
exit 0
