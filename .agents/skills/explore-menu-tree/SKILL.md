---
name: nhl-menu-navigator
description: >-
  Navigate NHL Legacy Recomp menus with image-recognition guidance. Two modes:
  EXPLORE to map menu trees / build menu graphs / catalog screens, and EXECUTE
  to perform goal-oriented operations (load rosters, trade players, save files,
  configure settings). Covers launch, daemon management, screenshot capture,
  and game-state interpretation through a vision model.
---

# nhl-menu-navigator

This skill builds on `run-nhl-smoke-test` to drive NHL Legacy Recomp menus
dynamically: take a screenshot, interpret it with a vision model, decide the
next input, execute it, repeat.

**Two modes** share the same infrastructure but have different loops and
termination conditions. The parent agent MUST commit to one mode before any
input.

## Mode Selection (MANDATORY)

Before launching the game, classify the user's request. State your chosen
mode in the conversation and justify it.

| Cue | Mode |
|-----|------|
| "map", "catalog", "explore", "find all options", "what screens exist", "build menu graph" | **EXPLORE** |
| "load", "trade", "save", "configure", "set", "change", "navigate to X and do Y" | **EXECUTE** |

A request containing both (e.g. "do a trade and write findings to the map")
is an EXECUTE task that also updates the persistent map. Choose EXECUTE.

## Persistent Reference: `map.md`

`map.md` at the repo root is a **persistent, cross-run artifact** — it
accumulates across sessions. It is the agent's map and compass, not a
per-run journal.

- **EXPLORE mode writes to it** heavily (menu graph, screen catalog).
- **EXECUTE mode reads it** for navigation planning and **appends gotchas**
  discovered during execution.

See `map.md` for its expected structure.

---

## Shared Infrastructure

### Prerequisites

- **Build**: `cargo build --workspace`
- **uinput** (`/dev/uinput`): group `input`, same as smoke test
- **Vision model**: image-capable model to interpret screenshots

### 1. Start the game

```
nohup bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh > screenshots/nhl-launch.log 2>&1 &
```

**IMPORTANT:** This command returns immediately. Proceed immediately.

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

Discover the window substring to use. Pass it to the daemon as
`--window-substring`. The daemon handles window matching; `--send` commands
don't need it.

### 4. Generate a run ID

All screenshots for this session land in the same directory:

```
RUN_ID=$(date +%Y%m%d_%H%M%S)_run
```

The screenshot directory will be `screenshots/$RUN_ID/`.

### 5. Start the daemon

The daemon keeps the virtual Xbox controller alive between commands,
eliminating the 3s warmup on every step. Start it once in the background:

```
nohup ./target/debug/nhl-input --daemon \
  --run-id "$RUN_ID" \
  --window-substring "nhllegacy" \
  --watch screenshots/latest.png \
  > screenshots/daemon.log 2>&1 &
```

Wait for initialization:

```
sleep 5
grep -q "ready for commands" screenshots/daemon.log && echo "daemon ready"
```

All subsequent steps use `--send` to dispatch commands to this daemon.

### 6. Vision interpretation (shared prompt)

**The parent MUST NOT read or attach screenshots.** A subagent interprets
each screenshot using this prompt and returns a single JSON object. No
markdown fences, no explanations.

```
Look at this screenshot file: <PATH>

You are navigating the menu system of an NHL hockey video game. Look at this
screenshot and respond ONLY with a single JSON object. No markdown fences, no
explanations.

Detect the layout type first, then fill in the appropriate fields.

For simple menus (single column of options):
{"screen": "<title or context, e.g. Main Menu, Settings, Pause Menu>",
 "layout": "list",
 "options": ["option1", "option2", ...],
 "selected": "<currently highlighted/selected option>",
 "gameplay": false,
 "nav_hints": ["<button prompts visible on screen>"],
 "confidence": "high|medium|low"}

For complex layouts (multiple columns, tabs, split screens, multi-panel):
{"screen": "<title or context>",
 "layout": "two_column|tabs|grid|custom",
 "regions": [
   {"name": "<descriptive name, e.g. 'Left Team Roster', 'Tab Bar', 'Settings Panel'>",
    "options": ["item1", "item2", ...],
    "selected": "<highlighted item, or empty string if none>"}
 ],
 "description": "<free-form explanation: what each region is, the overall layout, and the current navigation state>",
 "gameplay": false,
 "nav_hints": ["<button prompts visible on screen>"],
 "confidence": "high|medium|low"}

The "layout" field must be one of:
- "list" — a single vertical list of options
- "two_column" — two side-by-side lists (e.g., trade screen)
- "tabs" — tab bar with content panels below
- "grid" — a grid of selectable items
- "custom" — anything else (use "description" to explain it)

IMPORTANT: For complex layouts, always use "regions" to describe each distinct
area and "description" to explain the overall layout. List ALL items visible in
each region (at least what's on screen, even if many).

CRITICAL: If you see an ice rink, players on ice, a puck, or crowd — set
"gameplay": true and leave all other fields empty.
```

---

## EXPLORE Mode

### Purpose

Map the menu system: catalog screens, record transitions, build a menu graph.
Terminates when the agent determines the exploration goal is met (e.g. "all
reachable screens visited").

### Exploration loop

#### 4a. Observe the current screen

```
./target/debug/nhl-input --send 'screenshot("observe");'
```

The screenshot lands in `screenshots/$RUN_ID/<NNN>_observe.png`.

#### 4b. Delegate interpretation to a subagent

Use the shared vision prompt above. The subagent returns a JSON description.

#### 4c. Record the transition in state.json

Read the current `state.json` from the run directory (create if first
iteration). Append a transition record:

```json
{
  "from": "main_menu",
  "action": "dpad_down",
  "to": "game_modes",
  "screenshot": "screenshots/20260710_120000_run/001_observe.png",
  "confidence": "high"
}
```

Write the updated `state.json` back. This file is your **only** memory of
prior iterations — do not rely on conversation history.

#### 4d. Decide and execute the next action

Based on the subagent's JSON response, run one navigation step (no warmup
needed — the daemon keeps the controller alive):

```bash
# Select the highlighted option
./target/debug/nhl-input --send 'tap("a"); wait(2.5); screenshot("step_N");'

# Scroll down one item
./target/debug/nhl-input --send 'tap("dpad_down"); wait(0.5); screenshot("step_N");'

# Go back to previous screen
./target/debug/nhl-input --send 'tap("b"); wait(1.5); screenshot("step_N");'

# Move left/right between tabs
./target/debug/nhl-input --send 'tap("dpad_right"); wait(1.0); screenshot("step_N");'
```

Wait times:
- **Menu item scroll**: 0.5s
- **Screen transition (A to enter)**: 2.0–3.0s (loading screens)
- **Screen transition (B to go back)**: 1.5–2.0s
- **Game startup/title screens**: up to 10s

#### 4e. Repeat

Go back to 4a until the exploration goal is met.

### Output: Compile menu graph

After (or during) exploration, compile a menu graph from `state.json` and
update `map.md`:

- Each unique screen name → a node
- Each `(from, action, to)` triple → a directed edge
- Edge labels: the button pressed (e.g. "A", "dpad_down")

Update `map.md` with new screens and edges. Also append any operational
gotchas discovered (unexpected dialogs, loading quirks, etc.).

---

## EXECUTE Mode

### Purpose

Perform a concrete operation described by the user: load a roster, trade
players, save a file, configure a setting. The session MUST NOT terminate
until the goal is achieved and verified.

### Step 0: Parse the goal (MANDATORY — before any game action)

Break the user's request into an ordered task list and write it to
`screenshots/$RUN_ID/goal.json`. The agent MUST write this file before
sending any inputs to the game.

```json
{
  "goal": "<user's stated end state>",
  "tasks": [
    {
      "id": "<short_kebab_case_label>",
      "description": "<human-readable description>",
      "pre_screen": "<screen name required before this task>",
      "actions": ["<high-level action 1>", "<high-level action 2>"],
      "post_screen": "<screen name expected after this task succeeds>",
      "status": "pending"
    }
  ],
  "completion_gate": false
}
```

Granularity: break into the coarsest logical steps that each have a
distinguishable start and end screen. The vision-guided loop handles finer
navigation within each step.

Example for "load ROSTER1, trade 2 TBL for 2 FLA, save as TRADED_ROSTER":

```json
{
  "goal": "Trade 2 TBL players for 2 FLA players and save roster as TRADED_ROSTER",
  "tasks": [
    {
      "id": "load_roster1",
      "description": "Load ROSTER1",
      "pre_screen": "Roster Management",
      "actions": ["navigate to Load Roster", "select ROSTER1", "confirm"],
      "post_screen": "Roster Management (ROSTER1 loaded)",
      "status": "pending"
    },
    {
      "id": "execute_trade",
      "description": "Trade 2 TBL players for 2 FLA players via Roster Moves",
      "pre_screen": "Roster Management",
      "actions": ["navigate to Roster Moves", "select TBL", "pick 2 players", "select FLA", "pick 2 players", "confirm trade"],
      "post_screen": "Roster Moves (trade confirmed / updated rosters visible)",
      "status": "pending"
    },
    {
      "id": "save_roster",
      "description": "Save roster as TRADED_ROSTER",
      "pre_screen": "Roster Management",
      "actions": ["navigate to Save Roster", "enter name TRADED_ROSTER", "confirm save"],
      "post_screen": "Roster Management (TRADED_ROSTER saved confirmation visible)",
      "status": "pending"
    }
  ],
  "completion_gate": false
}
```

### Step 1: Navigate to the first task's pre-screen

Use EXPLORE mode techniques (or `map.md`) to reach the starting screen for
task 1. The daemon and vision pipeline are already running.

### Step 2: Execution loop

For **each task** in `goal.json` (in order):

#### 2a. PRE-CHECK

Take a screenshot, run the vision subagent, and confirm you are on the
expected `pre_screen`. If not, navigate to it first.

#### 2b. EXECUTE

Perform the planned actions. Use vision guidance for fine-grained navigation
within the step. The daemon keeps the controller alive — no warmup needed.

#### 2c. POST-CHECK (MANDATORY)

Take a screenshot, run the vision subagent, and confirm the screen matches
the expected `post_screen`.

- **If it matches:** Mark the task `"status": "completed"` in `goal.json`.
- **If it does NOT match:** Retry the task once. If it fails again, abort
  with a diagnostic: report the expected screen vs. the actual screen the
  vision model returned. Do NOT clean up — preserve the game/daemon state
  for debugging.

Update `goal.json` on disk after every status change.

### Step 3: Completion Gate (MANDATORY)

When all tasks are marked `"completed"`:

1. **Confirm**: Re-read `goal.json`. Is every task `"status": "completed"`?
2. **Final screenshot**: Take one last screenshot.
3. **Verify end state**: Run the vision subagent. Does the screen match the
   user's intended end state?
4. **If yes**: Set `"completion_gate": true` in `goal.json`. Report success
   to the user with evidence (relevant screenshot paths and what they show).
   Only then proceed to cleanup.
5. **If no**: Do NOT clean up. Report the discrepancy to the user and abort.

### Anti-drift rule

If at any point the vision subagent's JSON reads like it is cataloging menu
options rather than confirming a pre/post condition, **re-read `goal.json`**
and re-anchor to the current task. The agent is drifting into EXPLORE mode.

### Update map.md

After completion, append any operational gotchas to `map.md` (unexpected
dialogs, navigation quirks, wait time adjustments).

---

## Safety Rules (both modes)

| Rule | Why |
|------|-----|
| **If subagent returns `"gameplay": true`** | Press Start to open the pause screen, then follow the on-screen menus to quit back to the main menu |
| **Use A to select, B to go back** | Convention for NHL Legacy menus |
| **D-pad for navigation, not left stick** | Menus are grid-based; d-pad gives precise one-item moves. Left stick can overshoot |
| **Pause between inputs** | Allow at least 0.3–0.5s between taps to let the game register each input |
| **Keep alive** | If idle > 15s, tap `"dpad_down"` then `"dpad_up"` to prevent attract/demo mode |

## Cleanup

```
kill $(pgrep -f "nhl-input --daemon") 2>/dev/null; sleep 1
./scripts/kill-nhl.sh
```

**Before cleanup, verify the termination condition:**
- EXPLORE mode: exploration goal is met + `state.json` compiled
- EXECUTE mode: `goal.json` → `completion_gate: true`

## Troubleshooting

### Inputs not reaching the game

1. Check the daemon is alive: `pgrep -f "nhl-input --daemon"`
2. Verify the daemon has a controller running: look for "Microsoft X-Box One pad" in `evtest` output
3. If the daemon crashed, restart it with step 5 (shared infrastructure)
4. The daemon performs a 3s warmup at startup — no per-command warmup is needed

### Optional: FPS cap with MangoHud

To reduce CPU/GPU load during extended sessions, cap the game to ~10 fps:

1. Install: `sudo apt install mangohud` (Debian/Ubuntu) or `sudo dnf install mangohud` (Fedora)
2. Launch with the capped wrapper: `scripts/launch-nhl-capped.sh`
3. If MangoHud doesn't apply, try `strangle`:

```
strangle 10 bash ~/code/nhl-legacy/NHL\ Legacy\ Recomp/launch-nhl-legacy.sh
```

**Tap duration at low FPS:** The default `tap()` holds for 200ms. At 10fps
(100ms/frame), this spans ~2 frames. If inputs are dropped, use
`tap_ms("btn", 300)` or `hold("btn", 0.3)`.

### Screenshots are black or wrong window

Re-run `--list-windows` — the game may have spawned a different window.

## References

| Doc | Covers |
|-----|--------|
| `docs/scripting.md` | Full Rhai API: buttons, axes, functions, examples |
| `docs/window-discovery.md` | Window matching mechanics, known substrings |
| `docs/setup.md` | uinput permission setup |
| `AGENTS.md` | Build/lint/test commands, system deps |
| `scripts/kill-nhl.sh` | Kill the Proton game process tree |
| `scripts/launch-nhl-capped.sh` | FPS-capped game launcher (MangoHud wrapper) |
| `map.md` | Persistent menu reference: graph, screens, gotchas |
