---
name: explore-menu-tree
description: >-
  Map in-game menu trees by driving NHL Legacy Recomp with image-recognition
  guidance. Use this when exploring the game's UI, cataloging menu screens,
  discovering navigation paths, or building menu-graph data. The skill covers
  launch, ad-hoc input execution via --eval, screenshot capture, and game-state
  interpretation through a vision model.
---

# explore-menu-tree

This skill builds on `run-nhl-smoke-test` to systematically map the NHL Legacy
Recomp menu system. Instead of a fixed A+Start spam loop, the agent drives
navigation dynamically: take a screenshot, interpret it with a vision model,
decide the next input, execute it, repeat.

## Prerequisites

- **Build**: `cargo build --workspace`
- **uinput** (`/dev/uinput`): group `input`, same as smoke test
- **Vision model**: an image-capable model (MiMo2.5, GPT-4o, Claude, etc.) to interpret screenshots

## Checklist

Run these commands in order. Do not ask for confirmation.

### 1. Start the game

```
nohup bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh > screenshots/nhl-launch.log 2>&1 &
```

**IMPORTANT:** This command returns immediately. Do NOT wait for the game to finish loading. Proceed to the next sub-step right away.

```
sleep 15
```

```
ps aux | grep nhllegacy.exe | grep -v grep
```

You should see `nhllegacy.exe`. Continue to step 2.

### 2. Verify uinput access

```
ls -l /dev/uinput
```

Expected: group is `input` (`crw-rw----+ 1 root input 10, 223 ...`).

### 3. Discover the game window

```
./target/debug/nhl-input --list-windows
```

Use `"nhllegacy"` as the `--window-substring` for every invocation below.

### 4. Explore cycle

This is the core loop. The agent runs a vision-guided feedback loop:

#### 4a. Observe the current screen

```
./target/debug/nhl-input \
  -e 'screenshot("observe");' \
  --window-substring "nhllegacy"
```

This captures a screenshot into the timestamped run directory. The path is
printed on stderr. Also use `--watch screenshots/latest.png` if you want a
stable path to read back from.

#### 4b. Interpret the screenshot

Feed the screenshot to an image-recognition model. Request **JSON-only output**
(no prose, no markdown fences) to keep agent context small (see section 6 for
the full context-management strategy):

> You are navigating the menu system of an NHL hockey video game. Look at this
> screenshot and respond ONLY with a single JSON object. No markdown fences, no
> explanations.
> ```json
> {"screen": "<title or context, e.g. Main Menu, Settings, Pause Menu>",
>  "options": ["option1", "option2", ...],
>  "selected": "<currently highlighted/selected option>",
>  "gameplay": false,
>  "nav_hints": ["<button prompts visible on screen>"],
>  "confidence": "high|medium|low"}
> ```
> CRITICAL: If you see an ice rink, players on ice, a puck, or crowd — set
> "gameplay": true and leave all other fields empty.

#### 4c. Decide and execute the next action

Based on the model's response, run one navigation step. Examples:

```bash
# Select the highlighted option
./target/debug/nhl-input -e 'tap("a"); wait(2.5); screenshot("step_N");' --window-substring "nhllegacy"

# Scroll down one item
./target/debug/nhl-input -e 'tap("dpad_down"); wait(0.5); screenshot("step_N");' --window-substring "nhllegacy"

# Go back to previous screen
./target/debug/nhl-input -e 'tap("b"); wait(1.5); screenshot("step_N");' --window-substring "nhllegacy"

# Move left/right between tabs
./target/debug/nhl-input -e 'tap("dpad_right"); wait(1.0); screenshot("step_N");' --window-substring "nhllegacy"
```

Wait times are important:
- **Menu item scroll**: 0.5s (menus respond quickly)
- **Screen transition (A to enter)**: 2.0–3.0s (loading screens)
- **Screen transition (B to go back)**: 1.5–2.0s
- **Game startup/title screens**: up to 10s for initial loading

#### 4d. Repeat

Go back to 4b with the new screenshot until the exploration goal is met.

### 5. Safety rules

Violating these rules can send the agent into on-ice gameplay, which is
resource-intensive and derails menu mapping.

| Rule | Why |
|------|-----|
| **If model returns `"gameplay": true`** | Press Start to open the pause screen, then follow the on-screen menus to quit back to the main menu |
| **Use A to select, B to go back** | This is the convention for NHL Legacy menus |
| **D-pad for navigation, not left stick** | Menus are grid-based; d-pad gives precise one-item moves. Left stick can overshoot |
| **Pause between inputs** | Allow at least 0.3–0.5s between taps to let the game register each input (menus run at the game's native framerate) |
| **Keep alive** | If idle > 15s, tap `"dpad_down"` then `"dpad_up"` to prevent attract/demo mode |

### 6. Recording results & context management

#### State file (avoid context bloat)

The explore cycle accumulates context quickly: each iteration adds a screenshot,
the vision model's full response, and the bash command output. After ~5 cycles
the agent's context is bloated and reasoning degrades. Use these strategies:

**Primary: structured state file + JSON model output.**

Maintain a `state.json` in the run directory. Step 4b already requests JSON-only
output from the vision model — after each cycle, take that structured response
and append a compact transition record to the state file:

```json
{
  "from": "main_menu",
  "action": "dpad_down",
  "to": "play_now",
  "screenshot": "screenshots/20260709_120000_run/step_005.png",
  "confidence": "high"
}
```

At the start of each iteration, the agent reads `state.json` to know where it
is and what came before — it does NOT scroll through prior conversation messages.

**Fallback: fresh subagent per iteration.**

If context still grows too fast, delegate each decision to a fresh subagent. The
parent writes the current `state.json`, launches a subagent that reads the state
file + sees the latest screenshot + decides one action + writes back to the
state file. The parent only sees the action result and the updated graph — raw
model responses never reach the parent context.

#### Menu graph

After exploration completes (or incrementally during), compile a menu graph from
`state.json`:

- Each unique screen name maps to a node
- Each `(from, action, to)` triple maps to a directed edge
- Edge labels: the button pressed (e.g. "A", "dpad_down")

Store the compiled graph in the run directory (or report it to the user).

### 7. Kill the game

```
./scripts/kill-nhl.sh
```

## Troubleshooting

### Inputs not reaching the game after first --eval invocation

The `nhl-input` tool creates a virtual Xbox controller device on startup and
destroys it on exit. Short-lived `-e` invocations cause frequent device
plug/unplug. If the game doesn't react to inputs:

1. Add a 3s warm-up at the start of every `-e` script: `-e 'wait(3.0); tap("a"); ...'`
2. Verify the device exists mid-execution: `evtest /dev/input/event*` (look for "Microsoft X-Box One pad") while nhl-input is running
3. If the game consistently ignores inputs after hotplug, switch to a long-running script instead of per-step `-e` invocations

### Optional: FPS cap with MangoHud

To reduce CPU/GPU load during extended exploration, cap the game to ~10 fps:

1. Install: `sudo apt install mangohud` (Debian/Ubuntu) or `sudo dnf install mangohud` (Fedora)
2. Launch with the capped wrapper: `scripts/launch-nhl-capped.sh`
   (wraps the game with `mangohud --mangohud-config "fps_limit=10,no_display"`)
3. The `no_display` flag suppresses the MangoHud overlay so it doesn't clutter screenshots

If MangoHud doesn't apply to the Proton-launched game, try `strangle` instead:

```
strangle 10 bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh
```

### Screenshots are black or wrong window

Re-run `--list-windows` — the game may have spawned a different window. Use the
substring that appears in the list.

## References

| Doc | Covers |
|-----|--------|
| `docs/scripting.md` | Full Rhai API: buttons, axes, functions, examples |
| `docs/window-discovery.md` | Window matching mechanics, known substrings |
| `docs/setup.md` | uinput permission setup |
| `AGENTS.md` | Build/lint/test commands, system deps |
| `scripts/kill-nhl.sh` | Kill the Proton game process tree |
| `scripts/launch-nhl-capped.sh` | FPS-capped game launcher (MangoHud wrapper) |
