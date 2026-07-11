#!/usr/bin/env bash
# Guard gate: verifies log liveness before forwarding a --send command to
# the daemon. Prevents the agent from sending inputs without logging the
# previous step's vision result.
#
# Usage: ./scripts/safe-send.sh <run_id> <script>
#   run_id  - subdirectory name under screenshots/
#   script  - the Rhai script string to forward to --send
#
# Checks:
#   1. The last daemon screenshot event has a corresponding log entry
#   2. No daemon command was sent after the last screenshot without an
#      intervening log entry
#
# If all checks pass, forwards to: nhl-input --send <script>

set -euo pipefail

RUN_ID="${1:?Usage: safe-send.sh <run_id> <script>}"
shift
SCRIPT="$*"

DIR="screenshots/${RUN_ID}"

# Determine the nhl-input binary
NHL_INPUT="${NHL_INPUT:-./target/debug/nhl-input}"

# If there's no log file yet (very first step), allow
if [ ! -f "$DIR/run_log.jsonl" ]; then
    echo "SAFE-SEND: no log file yet, allowing first send"
    exec $NHL_INPUT --send "$SCRIPT"
fi

if [ ! -f "$DIR/daemon_events.jsonl" ]; then
    echo "SAFE-SEND: no daemon events file, allowing send"
    exec $NHL_INPUT --send "$SCRIPT"
fi

# Count screenshots in daemon events
SCREENSHOT_COUNT=$(grep -c '"event":"screenshot"' "$DIR/daemon_events.jsonl" 2>/dev/null || echo 0)
LOG_COUNT=$(wc -l < "$DIR/run_log.jsonl" 2>/dev/null || echo 0)

# The golden rule: every screenshot must have a log entry before the next input.
# Exception: the current script contains a screenshot() call (this is normal).
# We check: log_count >= screenshot_count - 1 (one screenshot may be pending)

if [ "$LOG_COUNT" -lt $((SCREENSHOT_COUNT - 0)) ]; then
    echo "SAFE-SEND: BLOCKED — unlogged screenshot(s) detected"
    echo "  screenshots: $SCREENSHOT_COUNT"
    echo "  log entries: $LOG_COUNT"
    echo "  Log every screenshot before sending more inputs."
    exit 1
fi

# Check that the last daemon event before this send was a screenshot,
# not a command (no blind input chaining)
LAST_EVENT=$(tail -1 "$DIR/daemon_events.jsonl" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('event',''))" 2>/dev/null || echo "")
if [ "$LAST_EVENT" = "command" ]; then
    echo "SAFE-SEND: BLOCKED — last daemon event was a command, not a screenshot"
    echo "  A command was sent without the resulting screenshot being logged."
    echo "  Run menu-vision and log-step before sending the next input."
    exit 1
fi

echo "SAFE-SEND: checks passed (screenshots=$SCREENSHOT_COUNT, logs=$LOG_COUNT)"
exec $NHL_INPUT --send "$SCRIPT"
