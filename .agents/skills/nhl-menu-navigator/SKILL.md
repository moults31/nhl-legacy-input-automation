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
nohup bash ./scripts/launch-nhl-capped.sh > screenshots/nhl-launch.log 2>&1 &
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
  --log-json \
  > screenshots/daemon.log 2>&1 &
```

Wait for initialization:

```
sleep 5
grep -q "ready for commands" screenshots/daemon.log && echo "daemon ready"
```

All subsequent steps use `--send` to dispatch commands to this daemon.

### 6. Vision interpretation (shared prompt)

> **HARD REQUIREMENT — READ THIS:**
>
> The parent agent CANNOT view images. Do NOT use `Read` on screenshots —
> it will fail. There is exactly ONE valid way to interpret a screenshot:
> the `menu-vision` subagent.
>
> | Mechanism | Allowed? |
> |-----------|----------|
> | `Task` tool with `subagent_type="menu-vision"` | **YES — only way** |
> | `Read` tool on `.png` file | **NO — will fail** |
> | Any other subagent type (`explore`, `general`, etc.) | **NO** |
> | Guessing / assuming what the screen shows | **NO — navigation will drift** |
>
> **No screenshot is ever interpreted without a `menu-vision` response.**
> If the `menu-vision` call fails or returns no valid JSON: **HALT.**
> Do not send any inputs. Retry the vision call or report the failure.

Pass the vision prompt below as the task description. The subagent returns a
single JSON object. No markdown fences, no explanations.

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

### 6a. Screenshot naming convention

**Screenshot labels must be neutral counters ONLY.** The label parameter
expresses nothing about screen identity — it is purely sequential. Screen
identity is assigned AFTER `menu-vision` returns.

| Allowed | Not allowed |
|---------|-------------|
| `screenshot("step_001")` | `screenshot("at_customize")` |
| `screenshot("step_014")` | `screenshot("tbl_player_selected")` |

Never label a screenshot based on intent or what you expect the screen to
show. You do not know what screen you are on until `menu-vision` confirms it.

### 6b. Mandatory per-step logging

After every `menu-vision` call, log one line to `screenshots/$RUN_ID/run_log.jsonl`
using the `nhl-input --log-step` tool. **THIS IS MANDATORY.** You are NOT
allowed to send a next input without logging the current step.

The tool validates the entry at write time — if it rejects the log, you have
a structural problem (missing vision fields, invalid JSON response, etc.) and
must HALT before sending any further inputs.

**Step 1 — write the exact vision prompt to a file:**

```bash
cat > /tmp/nhl_prompt.txt << 'PROMPT_EOF'
Look at this screenshot file: screenshots/$RUN_ID/NNN_step_NNN.png

I am executing a task in an NHL hockey video game. My goal.json task
EXPECTS this screenshot to show: <expected_screen>

Respond ONLY with a single JSON object. No markdown fences, no explanations.

Answer the MATCH question FIRST:

{"match": true|false, ...}
PROMPT_EOF
```

Copy the FULL text you sent to the `menu-vision` subagent. The single-quoted
heredoc (`<< 'PROMPT_EOF'`) means the shell will not expand variables or
escape special characters — the content is recorded verbatim.

**Step 2 — write the raw vision response to a file:**

```bash
cat > /tmp/nhl_response.txt << 'RESP_EOF'
{"match":true,"screen":"CUSTOMIZE","layout":"list","options":["CREATION ZONE",...],"selected":"CREATION ZONE","gameplay":false,"confidence":"high"}
RESP_EOF
```

Copy the EXACT JSON returned by `menu-vision`. Do not reformat, reorder,
or annotate. The tool will parse and validate it.

**Step 3 — log the step:**

```bash
nhl-input --log-step \
  --run-id "$RUN_ID" \
  --step <N> \
  --screenshot "screenshots/$RUN_ID/<NNN>_step_<N>.png" \
  --prompt-file /tmp/nhl_prompt.txt \
  --response-file /tmp/nhl_response.txt \
  --assessment "<match_confirmed|mismatch|recovery|halt>" \
  --decision "<navigate|recover|halt>" \
  --plan "<one-line summary of what input will be sent next and why>"
```

| Field | Valid values | Meaning |
|-------|-------------|---------|
| `--assessment` | `match_confirmed`, `mismatch`, `recovery`, `halt` | Agent's verdict after cross-checking vision response against goal.json |
| `--decision` | `navigate`, `recover`, `halt` | Whether to continue the plan, enter recovery, or stop |
| `--plan` | Free text (one line) | What input will be sent next and why |

**If `--log-step` exits non-zero: HALT.** Do not send any inputs. Rectify
the logging issue (missing fields, bad vision response, etc.) first.

### 6c. Log validation gate

Every 10 steps, run the validator:

```bash
./scripts/check-log.sh "$RUN_ID"
```

If it exits non-zero, the agent has been pressing buttons without logging.
**HALT** — catch up on the missing log entries before sending any further
inputs.

---

## EXPLORE Mode

### Purpose

Map the menu system: catalog screens, record transitions, build a menu graph.
Terminates when the agent determines the exploration goal is met (e.g. "all
reachable screens visited").

### Exploration loop

#### 4a. Observe the current screen

```
./target/debug/nhl-input --send 'screenshot("step_N");'
```

The screenshot lands in `screenshots/$RUN_ID/<NNN>_step_N.png`.

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
./target/debug/nhl-input --send 'scroll("down", 1, 300); screenshot("step_N");'
# Or for a single press: ./target/debug/nhl-input --send 'tap("dpad_down"); wait(0.5); screenshot("step_N");'

# Scroll down N items at once (within the daemon, no parallelism issues)
./target/debug/nhl-input --send 'scroll("dpad_down", 6, 300); screenshot("step_030");'

# Cycle team forward on the Player Movement trade screen (triggers are axes)
./target/debug/nhl-input --send 'tap_trigger("rt", 500); wait(0.4); screenshot("step_N");'

# Cycle league on the Player Movement trade screen (bumpers are buttons, use tap)
./target/debug/nhl-input --send 'tap("rb"); wait(0.7); screenshot("step_N");'

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
  `"match": false`. If both name and options match:
  1. **Log the result** — append to `screenshots/$RUN_ID/vision_log.jsonl`.
  2. Proceed to the NAVIGATION inner loop (3b).

- **If `"match": false`:** Log the discrepancy to
  `screenshots/$RUN_ID/discrepancies.jsonl` (see "Vision discrepancy
  tracking" below). Then navigate to the expected screen using the exact
  button path from `map.md` Navigation Reference. Do not proceed until
  both `"match": true` AND the options cross-check passes.

#### 3b. NAVIGATION inner loop

**CAUTION — daemon input discipline:** The daemon can silently drop inputs
when multiple `tap()` commands are batched into a single `--send` call. Send
**exactly one button press per `--send`** so every step produces a screenshot
you can verify. If you need ↓×6, send six separate `--send` commands.

**GATE — vision liveness check:** The `menu-vision` subagent (`subagent_type="menu-vision"`) is
the ONLY way to interpret a screenshot. The parent agent cannot view images.
If a `menu-vision` call fails, returns empty, or returns non-JSON: **HALT
immediately.** Do not press any buttons. The loop is: INPUT → SCREENSHOT →
menu-vision → (parse JSON) → next decision. **No menu-vision response = no
further inputs.**

For navigation within the task, follow this tight loop:

1. **ANCHOR**: Re-read `screenshots/$RUN_ID/goal.json`. Confirm the current
   task's `pre_screen` (start), `pre_options`, `post_screen` (destination),
   and `post_options`. Re-open `map.md` Navigation Reference. You are
   anchoring to known coordinates — do not guess from memory.

2. **CHECK**: Run the EXECUTE vision prompt via `menu-vision` with the
   expected screen. You MUST receive a valid JSON object. If the subagent
   call fails, returns empty, or returns non-JSON: **HALT.** Do not send
   any inputs. Retry the vision call. Only proceed to step 3 once you have
   a parseable JSON response. Check `"match"` and cross-check `"options"`
   against expected options. If mismatch, you are off course — press `B`
   to back up, re-read `goal.json`, and re-navigate from a known anchor
   in `map.md`.

3. **PLAN**: Consult the `map.md` Navigation Reference table for the shortest
   button path to the destination. If no entry exists, use the vision result
   to decide the next single d-pad step toward the target option.

4. **INPUT**: Send EXACTLY ONE button press plus wait + screenshot:

   ```bash
   # Face buttons and d-pad (use tap):
   ./target/debug/nhl-input --send 'tap("<button>"); wait(<time>); screenshot("step_N");'

   # Triggers (use tap_trigger — tap("rt") / tap("lt") will NOT work):
   ./target/debug/nhl-input --send 'tap_trigger("rt", 500); wait(0.5); screenshot("step_N");'

   # Multi-tap navigation (use scroll instead of for-loops):
   ./target/debug/nhl-input --send 'scroll("dpad_down", 6, 300); screenshot("step_N");'
   ```

   **IMPORTANT — Trigger discipline:** LT/RT are analog triggers, not digital
   buttons. `tap("rt")` and `tap("lt")` silently do nothing. Use `tap_trigger`
   or the explicit `set_axis` pattern. Valid trigger names: `"lt"`, `"rt"`,
   `"left_trigger"`, `"right_trigger"`.

   **IMPORTANT — scroll discipline:** `scroll(direction, count, delay_ms)`
   does N sequential taps with a 200ms hold per press, pausing `delay_ms`
   between releases. Valid directions: `"dpad_up"` / `"up"`, `"dpad_down"` /
   `"down"`, `"dpad_left"` / `"left"`, `"dpad_right"` / `"right"`.

   Never batch multiple inputs. The screenshot after each step is your only
   defense against drift.

5. **VERIFY**: Run the EXECUTE vision prompt via `menu-vision` on the new
   screenshot. Same halt rule as CHECK — you MUST receive a parseable JSON
   response before sending any further inputs. Log the result to
   `vision_log.jsonl`. Compare `"screen"` and `"options"` against the
   expected intermediate screen. If they don't match what the planned step
   should produce, treat as INTERRUPT (step 6).

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

  If both name and options match:
  1. **Log the result** — append to `screenshots/$RUN_ID/vision_log.jsonl`.
  2. Mark the task `"status": "completed"` in `goal.json`.

- **If `"match": false`:** Log the discrepancy to
  `screenshots/$RUN_ID/discrepancies.jsonl`. Then enter the **RECOVERY
  protocol** (step 3d). Do NOT skip tiers. Do NOT abort until you have
  exhausted all tiers.

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
- The last 5 lines of `screenshots/$RUN_ID/run_log.jsonl` (prompt, response, assessment, decision fields)
- The last 10 lines of `screenshots/$RUN_ID/daemon_events.jsonl` (if `--log-json` was used)

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

### Run log and diagnostics

The agent MUST maintain a single per-run log file: `screenshots/$RUN_ID/run_log.jsonl`.
This replaces the previous multi-file approach (`vision_log.jsonl`,
`discrepancies.jsonl`, `state.json`). Each line captures one decision cycle:
prompt sent, vision response, assessment, decision, and plan.

**`run_log.jsonl`** — one line per step, written via `nhl-input --log-step`:

```json
{"step":18,"screenshot":"screenshots/.../018_step_018.png",
 "vision_prompt":"Look at this screenshot...",
 "vision_response":{"match":true,"screen":"CUSTOMIZE","options":[...],"confidence":"high"},
 "assessment":"match_confirmed","decision":"navigate",
 "plan":"Press A to enter SAVE/LOAD/DELETE submenu"}
```

**`daemon_events.jsonl`** — written automatically by the daemon when started with
`--log-json`. Records command scripts and screenshot paths with timestamps:

```json
{"ts":"2026-07-10T23:07:22.872Z","event":"command","script":"tap(\"a\"); wait(2.0); screenshot(\"step_006\");"}
{"ts":"2026-07-10T23:07:27.469Z","event":"screenshot","path":"screenshots/.../006_step_006.png"}
```

These files are dumped as part of Tier 4 escalation diagnostics. They are
also the agent's best defense against drift — a gap between the screenshot
count and log entry count (detected by `check-log.sh`) signals that the
agent has been pressing buttons without introspection.

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
5. **Trigger inputs (LT/RT):** `tap("rt")` and `tap("lt")` silently do nothing
   because triggers are analog axes, not digital buttons. Use `tap_trigger("rt",
   500)` or the explicit `set_axis` pattern:
   ```
   set_axis("right_trigger", 1.0); wait(0.5); set_axis("right_trigger", 0.0); wait(0.4);
   ```
6. **Multi-tap batch inputs:** Never use shell `for` loops to send parallel
   `--send` commands — the daemon serializes them, causing timeouts. Use
   `scroll("dpad_down", 6, 300)` inside a single `--send` instead.

### Tap duration at low FPS:

The default `tap()` holds for 200ms. At 10fps (100ms/frame), this spans ~2 frames. If inputs are dropped, use
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
