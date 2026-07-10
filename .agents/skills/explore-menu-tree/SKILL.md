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

**The parent MUST NOT read or attach screenshots.** Use the Task tool with
`subagent_type="menu-vision"` to interpret each screenshot. Pass the vision
prompt below as the task description. The subagent returns a single JSON
object. No markdown fences, no explanations.

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

### EXECUTE-mode vision prompt (overrides shared prompt)

In EXECUTE mode, the vision subagent answers a **match question**, not a
cataloging question. The shared prompt above encourages freeform menu
cataloging, which causes the parent agent to drift into EXPLORE mode.
EXECUTE vision must compare the screenshot against an expected screen from
`goal.json`.

Use this template — substitute `<expected_screen>` with the current task's
`pre_screen` or `post_screen`:

```
Look at this screenshot file: <PATH>

I am executing a task in an NHL hockey video game. My goal.json task
EXPECTS this screenshot to show: <expected_screen>

Respond ONLY with a single JSON object. No markdown fences, no explanations.

Answer the MATCH question FIRST:

{"match": true|false,
 "screen": "<actual screen title/context>",
 "layout": "list|two_column|tabs|grid|custom",
 "options": ["<visible options>"],
 "selected": "<highlighted option>",
 "gameplay": false,
 "confidence": "high|medium|low",
 "actual_screen": "<only if match is false: what screen is this instead>"}

CRITICAL:
- If match is false, I must re-navigate. Accurate actual_screen is essential.
- Do NOT catalog options out of curiosity. Answer the match question first.
- If you see an ice rink, players, puck, or crowd: set "gameplay": true.
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

### PERSISTENCE — THE TASK IS NOT DONE UNTIL IT IS VERIFIED

This is the most important rule in the document:

- You SHALL NOT abort a task until every recovery tier has been exhausted.
- You SHALL NOT give up because menus "seem" wrong or vision "seems" confused.
- Vision models make mistakes. The game reacts unexpectedly. This is NORMAL.
- Your only valid reason to abort is that Tier 4 of the RECOVERY protocol
  has been reached AND Tier 3 was attempted at least twice.
- If you find yourself thinking "I encountered significant navigation
  difficulty" — that is a signal to enter the RECOVERY protocol, not to abort.
- "B to go back" solves 80% of navigation problems. Try it first.

Pressing B is ALWAYS safe: it moves you one screen back toward a known
anchor. When lost, press B before doing anything else.

### Step 0: Parse the goal (MANDATORY — before any game action)

Break the user's request into an ordered task list and write it to
`screenshots/$RUN_ID/goal.json`. The agent MUST write this file before
sending any inputs to the game.

**CRITICAL — writing pre_screen and post_screen:**

The vision model cannot reliably distinguish between screens at different
levels of the menu hierarchy that share the same name. For example,
"Roster Management" appears as:
- A list item in the CUSTOMIZE menu (alongside CREATION ZONE, SETTINGS, ...)
- The Roster Management submenu (TEAM ROSTERS, PLAYER MOVEMENT, ...)
- The SAVE/LOAD/DELETE screen (which has a Roster Management context header)

A single name like `"pre_screen": "Roster Management"` WILL cause `"match":
true` for all of these, sending you down the wrong path.

**Always include an `expected_options` field** listing the menu items that
should be visible. The vision model cross-checks this against what it sees.
Include both in the template:

```json
{
  "goal": "<user's stated end state>",
  "tasks": [
    {
      "id": "<short_kebab_case_label>",
      "description": "<human-readable description>",
      "pre_screen": "<screen title — be specific, e.g. 'ROSTER MANAGEMENT entry in CUSTOMIZE'>",
      "pre_options": ["<visible options that must appear on this screen>"],
      "actions": ["<high-level action 1>", "<high-level action 2>"],
      "post_screen": "<screen title expected after this task succeeds>",
      "post_options": ["<visible options that must appear on this screen>"],
      "status": "pending"
    }
  ],
  "completion_gate": false
}
```

Granularity: break into the coarsest logical steps that each have a
distinguishable start and end screen. The vision-guided loop handles finer
navigation within each step. If you don't know the exact options for a
screen, consult `map.md` or take an exploratory screenshot first.

Example with correct specificity:

```json
{
  "goal": "Trade 2 TBL players for 2 FLA players, save as TRADED_ROSTER",
  "tasks": [
    {
      "id": "load_roster",
      "description": "Load ROSTER1",
      "pre_screen": "CUSTOMIZE — ROSTER MANAGEMENT option is highlighted",
      "pre_options": ["CREATION ZONE", "CUSTOMIZE AI", "EA SPORTS MEDIA HUB", "FAVORITE TEAM", "OFFER CODE ENTRY", "PROFILE MANAGEMENT", "ROSTER MANAGEMENT", "SAVE/LOAD/DELETE", "SETTINGS"],
      "actions": ["Navigate to SAVE/LOAD/DELETE", "Select LOAD → ROSTERS", "Select ROSTER1 file", "Confirm Proceed"],
      "post_screen": "Load Roster — ROSTER1 loaded confirmation visible",
      "post_options": [],
      "status": "pending"
    },
    {
      "id": "execute_trade",
      "description": "Trade 2 TBL players for 2 FLA players",
      "pre_screen": "CUSTOMIZE — ROSTER MANAGEMENT option is highlighted",
      "pre_options": ["CREATION ZONE", "CUSTOMIZE AI", "EA SPORTS MEDIA HUB", "FAVORITE TEAM", "OFFER CODE ENTRY", "PROFILE MANAGEMENT", "ROSTER MANAGEMENT", "SAVE/LOAD/DELETE", "SETTINGS"],
      "actions": ["Navigate to ROSTER MANAGEMENT → PLAYER MOVEMENT", "Trade 2 TBL for 2 FLA via two-column trade screen", "Press X to execute"],
      "post_screen": "Roster Management submenu — trade confirmed",
      "post_options": ["TEAM ROSTERS", "PLAYER MOVEMENT", "EDIT LINES", "JERSEY NUMBERS", "SET DEFAULT ROSTERS", "DOWNLOAD ROSTERS"],
      "status": "pending"
    },
    {
      "id": "save_roster",
      "description": "Save roster as TRADED_ROSTER",
      "pre_screen": "CUSTOMIZE — SAVE/LOAD/DELETE option is highlighted",
      "pre_options": ["CREATION ZONE", "CUSTOMIZE AI", "EA SPORTS MEDIA HUB", "FAVORITE TEAM", "OFFER CODE ENTRY", "PROFILE MANAGEMENT", "ROSTER MANAGEMENT", "SAVE/LOAD/DELETE", "SETTINGS"],
      "actions": ["Navigate to SAVE", "Enter name TRADED_ROSTER", "Confirm save"],
      "post_screen": "CUSTOMIZE — save confirmation visible",
      "post_options": [],
      "status": "pending"
    }
  ],
  "completion_gate": false
}
```

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

### Step 1: Read map.md (MANDATORY — before any navigation)

**Open `map.md` at the repo root and read it.** You must memorize:
- The **Navigation Reference table** — exact button paths to each destination
- The **Menu Graph hierarchy** — parent/child relationships so you know where
  you are in the tree when vision gets confused
- The **Player Movement section** — if trading, know the two-column layout
  and which buttons switch panels/teams/leagues

If `map.md` has no entry for a destination you need, you MUST find and
record the path first (use EXPLORE mode techniques before EXECUTE tasks).

**Never navigate from memory or guess** — follow the exact button sequence
in the Navigation Reference table. Your only job is to execute that sequence
correctly, verifying with vision after each step.

### Step 2: Navigate to the first task's pre-screen

Use the button path from `map.md` Navigation Reference to reach the starting
screen for task 1. The daemon and vision pipeline are already running.

### Step 3: Execution loop

For **each task** in `goal.json` (in order):

#### 3a. PRE-CHECK

Take a screenshot, run the EXECUTE vision prompt with `<expected_screen>` set
to the task's `pre_screen`.

- **If `"match": true`:** Cross-check: do the returned `options` match the
  task's `pre_options`? If the options list is wrong, treat this as
  `"match": false`.

  If both name and options match, you are on the right screen. Proceed to
  the NAVIGATION inner loop (3b).

- **If `"match": false`:** Navigate to the expected screen using the exact
  button path from `map.md` Navigation Reference. Do not proceed until
  both `"match": true` AND the options cross-check passes.

#### 3b. NAVIGATION inner loop

**CAUTION — daemon input discipline:** The daemon can silently drop inputs
when multiple `tap()` commands are batched into a single `--send` call. Send
**exactly one button press per `--send`** so every step produces a screenshot
you can verify. If you need ↓×6, send six separate `--send` commands.

For navigation within the task, follow this tight loop:

1. **ANCHOR**: Re-read `screenshots/$RUN_ID/goal.json`. Confirm the current
   task's `pre_screen` (start), `pre_options`, `post_screen` (destination),
   and `post_options`. Re-open `map.md` Navigation Reference. You are
   anchoring to known coordinates — do not guess from memory.

2. **CHECK**: Run the EXECUTE vision prompt (see above) with the expected
   screen. Is `"match": true`? If not, you are off course — press `B` to
   back up, re-read `goal.json`, and re-navigate from a known anchor in
   `map.md`.

3. **PLAN**: Consult the `map.md` Navigation Reference table for the shortest
   button path to the destination. If no entry exists, use the vision result
   to decide the next single d-pad step toward the target option.

4. **INPUT**: Send EXACTLY ONE button press plus wait + screenshot:

   ```
   ./target/debug/nhl-input --send 'tap("<button>"); wait(<time>); screenshot("step_N");'
   ```

   Never batch multiple inputs. The screenshot after each step is your only
   defense against drift.

5. **VERIFY**: Run the EXECUTE vision prompt on the new screenshot. Did the
   screen advance toward `post_screen`?

6. **INTERRUPT**: If `"match"` is ever `false` or `"confidence"` is `"low"`,
   **stop sending inputs immediately**. Do not press more buttons trying to
   fix the situation — that drifts you further from the anchor.
   Press `B` ONCE to back up one screen. Take a screenshot and verify you
   are at a well-known screen. Re-enter the loop from step 1 (ANCHOR) with
   the current `goal.json` and `map.md`. Do NOT abort the task — INTERRUPT
   is a normal navigation recovery step, not a terminal state.

7. **REPEAT** from step 1 until `"match": true` on the `post_screen`,
   confirmed by Step 3c (POST-CHECK).

#### 3c. POST-CHECK (MANDATORY)

Take a screenshot, run the EXECUTE vision prompt with `<expected_screen>` set
to the task's `post_screen`.

- **If `"match": true`:** Cross-check: do the returned `options` match the
  task's `post_options`? The vision model can confuse similarly-named
  screens — e.g. "Roster Management" as a CUSTOMIZE list item vs. the Roster
  Management submenu itself. If the options list is wrong, treat this as
  `"match": false`.

  If both name and options match: mark the task `"status": "completed"` in
  `goal.json`.

- **If `"match": false`:** Enter the **RECOVERY protocol** (step 3d). Do NOT
  skip tiers. Do NOT abort until you have exhausted all tiers.

Update `goal.json` on disk after every status change.

#### 3d. RECOVERY protocol (when lost or match fails)

**YOU ARE NOT ALLOWED TO ABORT A TASK without exhausting all four tiers.**
Each tier must be attempted with screenshot verification before escalating.

**Tier 1 — One-step back**: Press `B`, wait 1.5s, screenshot. Run the EXECUTE
vision prompt with the current task's expected screen. Is this a screen you
recognize from `map.md`? If yes, re-plan from here. If still not matching,
go to Tier 2.

**Tier 2 — Return to anchor**: Press `B` repeatedly (up to 10 times, with
1.5s wait + screenshot after each) until you reach a well-known anchor:
Main Menu, CUSTOMIZE menu, or Roster Management submenu. Verify with the
vision prompt and cross-check with `map.md`. If you complete 10 presses
without reaching an anchor, go to Tier 3.

**Tier 3 — Full reset from Main Menu**: Press `Start` → navigate to "Quit"
→ "Main Menu" with the d-pad and `A`. Wait 5s for transition. Screenshot
to verify you are at the Main Menu. Re-read `map.md` Navigation Reference.
Re-navigate to the task's `pre_screen` from scratch, following the exact
button path from `map.md` — **never guess**. Once `pre_screen` is confirmed
by vision, retry the task. If Tier 3 fails, try it once more from the start.

**Tier 4 — Escalate**: If Tiers 1–3 all fail (at least two Tier 3 attempts),
dump a full diagnostic:
- Screenshot of the current screen
- The current `goal.json` contents
- The last 5 vision model JSON responses (the `match`, `screen`, `options`,
  and `actual_screen` fields)

Only then report the discrepancy to the user. **Do NOT clean up. Do NOT
kill the daemon or the game.** Preserve state for debugging.

### Step 4: Completion Gate (MANDATORY)

When all tasks are marked `"completed"`:

1. **Confirm**: Re-read `goal.json`. Is every task `"status": "completed"`?
2. **Final screenshot**: Take one last screenshot.
3. **Verify end state**: Run the EXECUTE vision prompt with the user's
   intended end state as `<expected_screen>`. Is `"match": true`?
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

### Termination gate (MANDATORY — re-read state file before any cleanup)

**EXECUTE mode:** Re-read `screenshots/$RUN_ID/goal.json`. Every task must
have `"status": "completed"` AND `"completion_gate"` must be `true`.

If NOT: **do NOT clean up.** You have not completed the task. Preserve the
game and daemon state, report the discrepancy, and abort. Destroying the
state destroys evidence the agent needs to retry.

**EXPLORE mode:** exploration goal is met + `state.json` compiled.

When the gate is satisfied, run:

```
kill $(pgrep -f "nhl-input --daemon") 2>/dev/null; sleep 1
./scripts/kill-nhl.sh
```

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
