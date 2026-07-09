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

Work through these steps in order. Each step includes its own verification.

### 1. Start the game

```sh
bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh &
```

Wait 15 seconds, then verify:

```sh
ps aux | grep nhllegacy.exe | grep -v grep
```

You should see `nhllegacy.exe`. The `.exe` suffix distinguishes the Proton
process from build tools that coincidentally reference "nhllegacy". If the
game is a native Linux build (unusual), drop the `.exe` suffix.

If the game isn't shown, do not loop — report the failure.

### 2. Verify uinput access

```sh
ls -l /dev/uinput
```

Must show `crw-rw----` with group `input`. See `docs/setup.md` if not.

### 3. Discover the game window

```sh
./target/debug/nhl-input --list-windows
```

Pick the most specific substring. `"nhllegacy"` is usually reliable. Use this
for every `--window-substring` flag below.

### 4. Focus the game window

Proton routes gamepad input to the focused window. Click on the game window
before running the script.

### 5. Run the smoke script (detached)

The tool runs as an infinite loop. Detach it from the shell session with
`setsid` so it survives when the parent terminal/tool session ends:

```sh
setsid ./target/debug/nhl-input \
  --script scripts/examples/smoke-to-main-menu.rhai \
  --window-substring "nhllegacy" \
  --watch screenshots/latest.png \
  > /tmp/smoke-run.log 2>&1 &
```

Note: do NOT use bare `&` without `setsid` — bash sends SIGHUP to background
processes when the parent shell exits, which kills the tool. `setsid` puts the
tool in its own session where it is immune to SIGHUP.

The tool creates a virtual Xbox controller, then taps A + Start every 2
seconds. It takes a labeled screenshot every 30 iterations (~60 seconds) into
`screenshots/<timestamp>_run/` and overwrites `screenshots/latest.png` after
each capture.

### 6. Verify progress

After 90 seconds, check if the game is advancing:

```sh
ls -la screenshots/*_run/
```

Compare file sizes of consecutive screenshots. Different sizes mean the game
screen changed — inputs are working. Identical sizes across two screenshots
means the game is stuck (see Troubleshooting).

For live viewing:
```sh
feh --reload 2 screenshots/latest.png
```

### 7. Wait for the main menu

The game reaches the main menu within ~3 minutes of A+Start spam. Stop when:
- The main menu is visible, OR
- 5 minutes have passed, OR
- Two consecutive screenshots have the same file size (game stuck)

### 8. Stop the script

```sh
kill $(pgrep -f "nhl-input.*smoke-to-main-menu")
```

If `pgrep` is unavailable, use `kill` with the PID from step 5, or press
Ctrl+C in the terminal where the tool is running.

### 9. Kill the game

**First, verify** which processes you're about to kill:
```sh
pgrep -af "nhllegacy.exe"
```

Confirm these are the game processes you started (not someone else's or
unrelated build processes), then kill them:

```sh
kill $(pgrep -f "nhllegacy.exe") 2>/dev/null
kill $(pgrep -f "steam.exe.*nhllegacy") 2>/dev/null
kill $(pgrep -f "python3.*proton.*nhllegacy") 2>/dev/null
```

If `pgrep` doesn't work in your environment, use PIDs directly from `ps aux`.

Alternatively, use the bundled helper (which does the same thing):
```sh
./scripts/kill-nhl.sh
```

### 10. Collect screenshots

Screenshots are in `screenshots/<timestamp>_run/`, named `smoke_<N>.png`.

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
- Check `/tmp/smoke-run.log` for the last screenshot — saved screenshots are
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
