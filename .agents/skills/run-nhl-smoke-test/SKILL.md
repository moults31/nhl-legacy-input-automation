---
name: run-nhl-smoke-test
description: >-
  Execute end-to-end smoke tests and operational recon runs against NHL Legacy
  Recomp via the nhl-input virtual Xbox controller tool. Use this whenever the
  user needs to automate the game to the main menu, run timed input sequences
  with periodic screenshots, troubleshoot automation failures, discover game
  windows, kill hung game processes, or perform any kind of operational recon
  of the automation pipeline. Even if the user doesn't say "smoke test"
  explicitly, you should use this skill when they ask about game input
  automation with nhl-input, NHL Legacy Recomp scripting, or Rhai-based
  controller automation.
---

# run-nhl-smoke-test

This repo provides a virtual Xbox controller (`nhl-input`) controlled by Rhai
scripts. The tool sends button presses to NHL Legacy Recomp (running via
Proton) and captures screenshots of the game window.

**Do NOT use `xdotool` or any external window-discovery tool.** The tool's
built-in `--list-windows` flag covers this entirely.

## Build

Build before the first script run (subsequent runs skip if code is unchanged):

```sh
cargo build --workspace
```

The binary is at `./target/debug/nhl-input`. Use it directly — not `cargo
run` — to avoid slower startup and process-lifecycle issues.

## Checklist

Run these commands in order. Do not ask for confirmation — proceed through each step.

### 1. Start the game

Launch the game fully detached so it runs in the background and this command returns immediately:

```sh
nohup bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh > screenshots/nhl-launch.log 2>&1 &
```

**IMPORTANT:** This command returns immediately. Do NOT wait for the game to finish loading. Proceed to the next sub-step right away.

Wait for Proton to boot:

```sh
sleep 15
```

Verify the game process exists:

```sh
ps aux | grep nhllegacy.exe | grep -v grep
```

You should see `nhllegacy.exe`. Continue to step 2.

### 2. Verify uinput access

```sh
ls -l /dev/uinput
```

Expected: group is `input` (e.g., `crw-rw----+ 1 root input 10, 223 ... /dev/uinput`).

Continue to step 3.

### 3. Discover the game window

```sh
./target/debug/nhl-input --list-windows
```

Use `"nhllegacy"` as the substring for every `--window-substring` flag below.

Continue to step 4.

### 4. Run the smoke script (detached)

```sh
setsid ./target/debug/nhl-input \
  --script scripts/examples/smoke-to-main-menu.rhai \
  --window-substring "nhllegacy" \
  --watch screenshots/latest.png \
  > screenshots/smoke-run.log 2>&1 &
```

Continue to step 5.

### 5. Verify screenshots

Wait long enough for at least 3 screenshots to be captured (they land at ~30s intervals):

```sh
sleep 95
```

Count the screenshots in the timestamped run dir:

```sh
ls screenshots/*_run/smoke_*.png 2>/dev/null | wc -l
```

You should see at least 3 files. Then consolidate logs into the run dir:

```sh
RUN_DIR=$(ls -td screenshots/*_run/ | head -1)
mv screenshots/smoke-run.log "$RUN_DIR/"
mv screenshots/nhl-launch.log "$RUN_DIR/" 2>/dev/null || true
echo "Run dir: $RUN_DIR"
ls -la "$RUN_DIR"
```

Continue to step 6.

### 6. Kill the game

```sh
./scripts/kill-nhl.sh
```

Continue to step 7.

### 7. Done

Tell the user: "Smoke test complete. Screenshots and logs are in `screenshots/<timestamp>_run/`."

## Troubleshooting

### Inputs not reaching the game (game stuck at startup screen)

1. **Is the game window focused?** Click on it.
2. **Does the virtual controller exist?** While tool is running:
   ```sh
   evtest /dev/input/event*
   ```
   Look for "Microsoft X-Box One pad".
3. **Controller discovery timing.** Kill everything and restart: run the tool
   FIRST (wait 3s), then start the game.

### Tool process died prematurely

If `ps aux | grep nhl-input` shows nothing but you didn't kill it:
- You probably ran it with bare `&` instead of `setsid`. The parent shell
  exited and sent SIGHUP. Restart with `setsid`.
- Check `screenshots/smoke-run.log` for the last screenshot — saved screenshots are
  not lost.

### Wrong window in screenshots

Rerun `--list-windows` and pick a more specific `--window-substring`. The
tool logs the matched title on every capture.

### Game won't die

```sh
# Find PIDs
ps aux | grep -E "nhllegacy.exe|steam.exe.*nhllegacy|python3.*proton.*nhllegacy" | grep -v grep
# Kill by PID
kill <pid1> <pid2> <pid3>
# If stubborn, escalate:
kill -9 <pid1> <pid2> <pid3>
```

## References

| Doc | Covers |
|-----|--------|
| `docs/scripting.md` | Full Rhai API: buttons, axes, functions, examples |
| `docs/window-discovery.md` | Window matching mechanics, known substrings |
| `docs/setup.md` | uinput permission setup |
| `AGENTS.md` | Build/lint/test commands, system deps |
| `scripts/kill-nhl.sh` | Kill the Proton game process tree |
