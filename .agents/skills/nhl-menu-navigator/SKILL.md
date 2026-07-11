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

**CRITICAL — RUN_ID naming discipline:**
The RUN_ID MUST use the neutral pattern `$(date +%Y%m%d_%H%M%S)_run` for
EXECUTE runs or `$(date +%Y%m%d_%H%M%S)_explore` for EXPLORE runs. Never
append descriptive suffixes like `_trade_mtl_tor` or `_load_roster`. Task
context belongs in `goal.json`, not in directory names. A descriptive
directory name leaks goal information to the vision model through the
`Screenshot:` header in the vision prompt and causes hallucination.

### 5. Start the daemon

The daemon keeps the virtual Xbox controller alive between commands,
eliminating the 3s warmup on every step. Start it once in the background:

```
nohup ./target/debug/nhl-input --daemon \
  --run-id "$RUN_ID" \
  --window-substring "nhllegacy" \
  --watch _screenshot.png \
  --log-json \
  > screenshots/daemon.log 2>&1 &
```

Wait for initialization:

```
sleep 5
grep -q "ready for commands" screenshots/daemon.log && echo "daemon ready"
```

All subsequent steps use `--send` to dispatch commands to this daemon.

**NEVER pass `--no-require-logging` to the daemon.** This flag disables the
step-logging enforcement gate and exists only for manual debugging outside
the navigator skill. The daemon enforces that every `--send` command is
preceded by a valid `--log-step` call for the previous screenshot. If you
forget to log a step, the daemon will reject your next command with an
error — that is a safety feature, not a bug.

Verify that logging enforcement is active:

```
sleep 1
grep -q "ready for commands" screenshots/daemon.log && echo "daemon ready"
```

If the daemon exits or fails to start, check `screenshots/daemon.log`. The
enforcement gate is active by default — no extra flags are needed.

### 6. Vision interpretation (shared prompt — used by both EXPLORE and EXECUTE)

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

The vision subagent is a **pure observer**. It receives no task context,
no expected screen, no goal information. It catalogues what it sees and
nothing more. The main agent owns all matching and decision-making.

Pass the unified vision prompt below as the task description. The subagent
returns a single JSON object. No markdown fences, no explanations.

```
Screenshot: _screenshot.png

Describe this screenshot from an NHL Legacy hockey game menu system.

Be exhaustive and precise. Report only what you observe — do not guess what
"should" be there. If any text is partially obscured, note it.

1. List ALL visible text strings — titles, menu items, breadcrumb trails,
   button hints, labels, body text. Leave nothing out.
2. Describe the visual layout in words — positions, panels, visual hierarchy.
3. Name the screen (the main title or heading visible). Note any breadcrumb
   trail (e.g. "CUSTOMIZE > CREATION ZONE") if visible.
4. List all selectable options and which one is highlighted.
5. Note any button hints (A, B, X, Y, LB, RB, LT, RT) on screen.

Respond ONLY with a single JSON object. No markdown fences.

{
  "all_text": ["every", "distinct", "text", "string", "on", "screen"],
  "screen_title": "Main title or heading text",
  "breadcrumbs": "Path trail if visible, e.g. CUSTOMIZE > CREATION ZONE, or empty string",
  "layout": "list|two_column|tabs|grid|custom",
  "layout_description": "Free-text description of visual arrangement",
  "options": ["selectable", "menu", "items"],
  "selected": "highlighted option, or empty string if none",
  "button_hints": ["A Select", "B Back"],
  "gameplay": false,
  "confidence": "high|medium|low",
  "regions": [
    {
      "name": "descriptive region label",
      "options": ["items", "in", "this", "region"],
      "selected": "highlighted item or empty string"
    }
  ]
}
```

**CRITICAL — Prompt format discipline:**
- The `Screenshot:` line at the top is the ONLY place the filename appears.
  It always reads `Screenshot: _screenshot.png` — the daemon's `--watch` flag
  keeps this file up to date with the latest screenshot. Never change this
  line or add any other text to the prompt.
- This prompt is the EXACT text you pass as the `prompt` parameter to
  the `Task` tool with `subagent_type="menu-vision"`. Copy it verbatim.
- `--log-step` will REJECT any prompt containing banned patterns:
  goal context ("I am executing a task", "EXPECTS"), wrong template
  openers ("Look at this screenshot file", "You are navigating"),
  or match questions ("MATCH question", "{\"match\"\:").
- If `--log-step` rejects the log due to prompt validation, you MUST
  fix the prompt and re-log before sending any further inputs.

#### Prompt variants: exhaustive vs. streamlined

The canonical prompt above (with `all_text`) is the **exhaustive** variant.
Use it for:
- EXPLORE mode (you need full screen cataloging)
- First-launch dialogs (unknown screens need full description)
- Any step where the screen is unfamiliar and not in `map.md`

For EXECUTE mode routine navigation between known screens (screens that
appear in `map.md`), use the **streamlined** variant instead — it drops
the exhaustive `all_text` requirement and focuses on structured fields
needed for matching and navigation:

```
Screenshot: _screenshot.png

Describe this NHL Legacy hockey game menu screenshot.
Report ONLY what you observe.

1. Name the screen (main title/heading). Note breadcrumbs if visible.
2. List all selectable options and which one is highlighted.
3. Note any visible button hints (A, B, X, Y, LB, RB, LT, RT).
4. Describe the visual layout.

Respond ONLY with a single JSON object. No markdown fences.

{
  "screen_title": "Main title or heading text",
  "breadcrumbs": "Path trail if visible, e.g. CUSTOMIZE > CREATION ZONE, or empty string",
  "layout": "list|two_column|tabs|grid|custom",
  "layout_description": "Free-text description of visual arrangement",
  "options": ["selectable", "menu", "items"],
  "selected": "highlighted option, or empty string if none",
  "button_hints": ["A Select", "B Back"],
  "gameplay": false,
  "confidence": "high|medium|low",
  "regions": [
    {
      "name": "descriptive region label",
      "options": ["items", "in", "this", "region"],
      "selected": "highlighted item or empty string"
    }
  ]
}
```

When using the streamlined prompt, relax Pass A check C2: if `all_text` is
absent or empty, skip C2 (the field was not requested). All other Pass A
and Pass B checks apply normally.
```

**Region examples for common layouts:**

```
// list layout (default for simple menus) — regions can be omitted:
{"layout": "list", "regions": null, ...}

// two_column layout (e.g. Player Movement trade screen) — REQUIRED, exactly 2 regions:
{
  "layout": "two_column",
  "regions": [
    {"name": "Left Panel", "options": ["ANA", "ARI", "BOS", ...], "selected": "ANA"},
    {"name": "Right Panel", "options": ["Free Agents", ...], "selected": "Free Agents"}
  ],
  ...
}

// tabs layout (e.g. Settings submenu) — REQUIRED, one region per tab:
{
  "layout": "tabs",
  "regions": [
    {"name": "Rules Tab", "options": ["Offsides", "Icing", ...], "selected": "Offsides"},
    {"name": "Gameplay Tab", "options": ["Game Speed", ...], "selected": "Game Speed"}
  ],
  ...
}
```

Field rules:
- `all_text`: every distinct text string on screen. Must include screen
  titles, menu items, breadcrumb trails, button hints, labels, and body
  text. Duplicates are OK but prefer deduplication.
- `screen_title`: the primary heading/title. Empty string if none visible.
- `breadcrumbs`: the breadcrumb trail if visible (e.g. "CUSTOMIZE > CREATION
  ZONE"), or empty string.
- `layout`: one of `list` (single vertical list), `two_column` (two
  side-by-side panels), `tabs` (tab bar with content), `grid` (tiled
  items), `custom` (anything else — explain in `layout_description`).
- `layout_description`: free-text. Where elements are positioned, how many
  panels/columns, visual hierarchy, what each region contains.
- `options`: all selectable menu items or list entries. Empty array if none
  (e.g. loading screen).
- `selected`: the highlighted/focused option. Empty string if nothing is
  highlighted or if the screen has no selectable items.
- `button_hints`: any button prompts visible on screen (e.g. "A Select",
  "B Back", "X Execute Move"). Empty array if none.
- `gameplay`: set to `true` ONLY if you see an ice rink, players on ice,
  a puck, or crowd. When `gameplay` is `true`, set `screen_title` to "",
  `options` and `all_text` to `[]`, and `layout` to `"custom"`.
- `confidence`: `high` if all text is clearly legible and the screen type
  is unambiguous. `medium` if some text is obscured or the identity is
  uncertain. `low` if the screen is heavily obscured, blurry, or you are
  guessing.
- `regions`: REQUIRED for `two_column`, `tabs`, and `grid` layouts.
  Omit for `list` layouts unless there are distinct non-list regions.
  Each region must have `name`, `options` (array of strings), and
  `selected` (string). For `two_column`, exactly 2 regions.
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

**How screenshots reach the subagent:** The daemon is started with
`--watch _screenshot.png` (see step 5). After every `screenshot()` call, the
daemon copies the latest image to `_screenshot.png` at the repo root. The
vision prompt always references this file. Your `--send` scripts still write
per-step screenshots to the run directory for diagnostics — the `--watch` file
is an always-up-to-date copy just for the subagent.

### 6b. Mandatory per-step logging

After every `menu-vision` call, log one line to `screenshots/$RUN_ID/run_log.jsonl`
using the `nhl-input --log-step` tool. **THIS IS MANDATORY.** You are NOT
allowed to send a next input without logging the current step.

The tool validates the entry at write time — if it rejects the log, you have
a structural problem (missing vision fields, invalid JSON response, etc.) and
must HALT before sending any further inputs.

#### Multi-input scripts are ALLOWED — inter-script verification is MANDATORY

You MAY batch multiple inputs in a single `--send` script for well-mapped
navigation sequences:

```bash
./target/debug/nhl-input --send 'scroll("dpad_down", 6, 300); wait(0.5); screenshot("step_N");'
```

The cycle is:

```
SCRIPT (ends with screenshot()) → menu-vision → log-step → next decision → SCRIPT → ...
```

NEVER:
- Send a script without a terminal `screenshot()` call
- Send a second script before the previous screenshot has been analyzed
  by `menu-vision` AND logged via `nhl-input --log-step`
- Chain two scripts back-to-back without vision+log between them
- Use multiple individual `tap("dpad_*")` calls when a known scroll count
  applies — the run log's `plan` field and the `--send` script must agree.
  If the plan says "↓×3", the script must use `scroll("dpad_down", 3, 300)`.
  Individual taps for repeat presses are a discipline violation.

#### Logging procedure

The prompt sent to `menu-vision` is the exact unified vision prompt from §6
with `Screenshot: _screenshot.png` (the daemon's `--watch` file). **No task
context, no expected screen, no goal.**

**Step 1 — write the exact vision prompt to a file:**

```bash
cat > /tmp/nhl_prompt.txt << 'PROMPT_EOF'
Screenshot: _screenshot.png

Describe this screenshot from an NHL Legacy hockey game menu system.

Be exhaustive and precise. Report only what you observe — do not guess what
"should" be there. If any text is partially obscured, note it.

1. List ALL visible text strings — titles, menu items, breadcrumb trails,
   button hints, labels, body text. Leave nothing out.
2. Describe the visual layout in words — positions, panels, visual hierarchy.
3. Name the screen (the main title or heading visible). Note any breadcrumb
   trail (e.g. "CUSTOMIZE > CREATION ZONE") if visible.
4. List all selectable options and which one is highlighted.
5. Note any button hints (A, B, X, Y, LB, RB, LT, RT) on screen.

Respond ONLY with a single JSON object. No markdown fences.

{
  "all_text": ["every", "distinct", "text", "string", "on", "screen"],
  "screen_title": "Main title or heading text",
  "breadcrumbs": "Path trail if visible, e.g. CUSTOMIZE > CREATION ZONE, or empty string",
  "layout": "list|two_column|tabs|grid|custom",
  "layout_description": "Free-text description of visual arrangement",
  "options": ["selectable", "menu", "items"],
  "selected": "highlighted option, or empty string if none",
  "button_hints": ["A Select", "B Back"],
  "gameplay": false,
  "confidence": "high|medium|low",
  "regions": [
    {
      "name": "descriptive region label",
      "options": ["items", "in", "this", "region"],
      "selected": "highlighted item or empty string"
    }
  ]
}
PROMPT_EOF
```

Copy the FULL text you sent to the `menu-vision` subagent. The single-quoted
heredoc (`<< 'PROMPT_EOF'`) means the shell will not expand variables or
escape special characters — the content is recorded verbatim.

**Step 2 — write the raw vision response to a file:**

```bash
cat > /tmp/nhl_response.txt << 'RESP_EOF'
{"all_text":["CUSTOMIZE","CREATION ZONE","CUSTOMIZE AI","...","A Select","B Back"],"screen_title":"CUSTOMIZE","breadcrumbs":"","layout":"list","layout_description":"Vertical menu list with header CUSTOMIZE at top, 9 options below, button hints bar at bottom","options":["CREATION ZONE","CUSTOMIZE AI","EA SPORTS MEDIA HUB","FAVORITE TEAM","OFFER CODE ENTRY","PROFILE MANAGEMENT","ROSTER MANAGEMENT","SAVE/LOAD/DELETE","SETTINGS"],"selected":"CREATION ZONE","button_hints":["A Select","B Back","Y Game Manual"],"gameplay":false,"confidence":"high"}
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
  --assessment "<goal_match|goal_mismatch|inconsistent|recovery|halt>" \
  --decision "<navigate|recover|halt>" \
  --plan "<one-line summary of what input will be sent next and why>"
```

| Field | Valid values | Meaning |
|-------|-------------|---------|
| `--assessment` | `goal_match`, `goal_mismatch`, `inconsistent`, `recovery`, `halt` | Agent's verdict after consistency checks + goal comparison |
| `--decision` | `navigate`, `recover`, `halt` | Whether to continue the plan, enter recovery, or stop |
| `--plan` | Free text (one line) | What input will be sent next and why |

Assessment meanings:
- `goal_match` — vision description is internally consistent AND matches goal.json expectations
- `goal_mismatch` — vision description is internally consistent but does NOT match the expected screen from goal.json
- `inconsistent` — vision description is self-contradictory (Pass A consistency checks failed)
- `recovery` — agent is executing the RECOVERY protocol
- `halt` — agent is stopping (Tier 4 escalation or task abort)

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

The vision model now returns a pure description — it has no knowledge of
your goal. The main agent must compare the vision response's `screen_title`,
`options`, and `layout_description` against goal.json fields to determine
if the screen matches expectations.

**Always include `pre_screen_title` and `post_screen_title`** — the canonical
screen name that the vision model's `screen_title` field should contain for
a match. Alongside `pre_options` and `post_options` for secondary verification.

```json
{
  "goal": "<user's stated end state>",
  "tasks": [
    {
      "id": "<short_kebab_case_label>",
      "description": "<human-readable description>",
      "pre_screen_title": "<canonical screen name from map.md — e.g. MAIN MENU, CUSTOMIZE, ROSTER MANAGEMENT>",
      "pre_screen": "<human-readable screen description>",
      "pre_options": ["<visible options that must appear on this screen>"],
      "actions": ["<high-level action 1>", "<high-level action 2>"],
      "post_screen_title": "<canonical screen name from map.md>",
      "post_screen": "<human-readable screen description>",
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
      "pre_screen_title": "CUSTOMIZE",
      "pre_screen": "CUSTOMIZE — ROSTER MANAGEMENT option is highlighted",
      "pre_options": ["CREATION ZONE", "CUSTOMIZE AI", "EA SPORTS MEDIA HUB", "FAVORITE TEAM", "OFFER CODE ENTRY", "PROFILE MANAGEMENT", "ROSTER MANAGEMENT", "SAVE/LOAD/DELETE", "SETTINGS"],
      "actions": ["Navigate to SAVE/LOAD/DELETE", "Select LOAD → ROSTERS", "Select ROSTER1 file", "Confirm Proceed"],
      "post_screen_title": "SAVE/LOAD/DELETE",
      "post_screen": "Load Roster — ROSTER1 loaded confirmation visible",
      "post_options": [],
      "status": "pending"
    },
    {
      "id": "execute_trade",
      "description": "Trade 2 TBL players for 2 FLA players",
      "pre_screen_title": "CUSTOMIZE",
      "pre_screen": "CUSTOMIZE — ROSTER MANAGEMENT option is highlighted",
      "pre_options": ["CREATION ZONE", "CUSTOMIZE AI", "EA SPORTS MEDIA HUB", "FAVORITE TEAM", "OFFER CODE ENTRY", "PROFILE MANAGEMENT", "ROSTER MANAGEMENT", "SAVE/LOAD/DELETE", "SETTINGS"],
      "actions": ["Navigate to ROSTER MANAGEMENT → PLAYER MOVEMENT", "Trade 2 TBL for 2 FLA via two-column trade screen", "Press X to execute"],
      "post_screen_title": "ROSTER MANAGEMENT",
      "post_screen": "Roster Management submenu — trade confirmed",
      "post_options": ["TEAM ROSTERS", "PLAYER MOVEMENT", "EDIT LINES", "JERSEY NUMBERS", "SET DEFAULT ROSTERS", "DOWNLOAD ROSTERS"],
      "status": "pending"
    },
    {
      "id": "save_roster",
      "description": "Save roster as TRADED_ROSTER",
      "pre_screen_title": "CUSTOMIZE",
      "pre_screen": "CUSTOMIZE — SAVE/LOAD/DELETE option is highlighted",
      "pre_options": ["CREATION ZONE", "CUSTOMIZE AI", "EA SPORTS MEDIA HUB", "FAVORITE TEAM", "OFFER CODE ENTRY", "PROFILE MANAGEMENT", "ROSTER MANAGEMENT", "SAVE/LOAD/DELETE", "SETTINGS"],
      "actions": ["Navigate to SAVE", "Enter name TRADED_ROSTER", "Confirm save"],
      "post_screen_title": "CUSTOMIZE",
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

**First-launch dialog handling:** After pressing Start on the title screen,
the game may present zero or more first-launch dialogs (Autosave Information,
Profile Warning, Choose Your Favorite Team, Tutorial prompt). See `map.md`
"First Launch Hazards" for recognition and dismissal instructions. These
dialogs can appear in any order or not at all. After each `A`-press to
dismiss, take a screenshot and verify with vision before sending the next
input.

### Step 3: Execution loop

For **each task** in `goal.json` (in order):

#### 3a. PRE-CHECK

Take a screenshot, run the unified vision prompt via `menu-vision`, and log
the result via `nhl-input --log-step`.

Then perform two passes:

**Pass A — Internal consistency (mandatory — before any goal comparison):**

| # | Check | If fails |
|---|-------|----------|
| C1 | `selected` (if non-empty) appears in `options` | Assessment → `inconsistent`. Trigger INTERRUPT. |
| C2 | Every string in `options` and `screen_title` appears in `all_text` (fuzzy, case-insensitive substring match). At minimum: `options` items and `screen_title` must each be contained somewhere in `all_text`. | Assessment → `inconsistent`. <br>**Skip C2** if `all_text` is empty or absent (streamlined prompt — `all_text` was not requested). |
| C3 | If `breadcrumbs` is non-empty, the last segment of the breadcrumb trail should contain `screen_title` (case-insensitive). | Downgrade confidence one level. |
| C4 | `layout_description` must be semantically consistent with `layout`. For example: if `layout` is `"list"` but `layout_description` says "two side-by-side panels", that's a contradiction. | Downgrade confidence one level. |
| C5 | If `layout` is `"list"` and `regions` is present, regions are harmless. If `layout` is `"two_column"`, `"tabs"`, or `"grid"`, `regions` MUST be present with options that collectively account for what's in `options`. | Downgrade confidence one level. |
| C6 | `button_hints` should be plausible for the claimed screen. E.g. "X Execute Move" on a main menu is implausible, "A Select" / "B Back" is universal and always plausible. | Downgrade confidence one level. |
| C7 | If `confidence` is `"high"` but 3+ of C1–C6 fail, override `confidence` to `"low"`. | Trigger INTERRUPT. |

If Pass A produced `confidence: "low"` or triggered INTERRUPT: treat as the
vision response being unreliable. Do NOT compare against goal.json yet.
Enter RECOVERY protocol (§3d).

**Pass B — Goal matching (only after Pass A passes):**

Compare the vision response against the task's `pre_screen_title`,
`pre_options`, and `pre_screen` from `goal.json`:

1. **Screen title match**: Does `vision.screen_title` contain (case-insensitive
   substring) the task's `pre_screen_title`?
2. **Options match**: Do `vision.options` contain the task's `pre_options`
   (fuzzy — at least 70% overlap)?
3. **Layout match**: Does `vision.layout` match the expected layout for this
   screen (from `map.md` or task context)?

- **If all three pass**: Assessment → `goal_match`. Proceed to §3b
  (NAVIGATION inner loop).
- **If any fails**: Assessment → `goal_mismatch`. Before entering RECOVERY,
  run the **hallucination guard** (§3a.1) to rule out vision model fabrication.
  If the guard clears suspicion, navigate to the expected screen using the
  exact button path from `map.md` Navigation Reference. Do not proceed until
  both Pass A and Pass B pass.

#### 3a.1. Hallucination guard (before recovery from a mismatch)

Vision models can fabricate screen titles and options with high confidence.
Before entering the RECOVERY protocol for a Pass B failure, check for
hallucination:

1. **Sanity check**: Do `vision.screen_title` and at least half of
   `vision.options` match any screen documented in `map.md`?
   - Quick scan: if the screen title mentions menus that don't exist in the
     map (e.g. "GAME MODES", "GAME STOP", "ONLINE SOCIETY"), or options
     contain made-up items not in any map.md node, the response is suspect.
   - If the screen IS known but doesn't match `goal.json` expectations, the
     agent is simply off course → proceed to RECOVERY normally.

2. **If the screen is unrecognizable** (title and options don't match any
   `map.md` node AND the title seems fabricated):
   - **Do NOT send any input.** Re-take a screenshot with the same label:
     ```bash
     ./target/debug/nhl-input --send 'screenshot("step_N");'
     ```
   - Run the vision prompt again via `menu-vision` on the new screenshot.
   - If the second vision call returns the **same** unrecognizable screen
     (matching title + similar options), the screen is real. Log both calls
     and enter RECOVERY.
   - If the second vision call returns a **different** screen (one that is
     recognizable or matches goal.json), the first call was a hallucination.
     Log the original call as `inconsistent` and continue with the corrected
     vision result. Discard the hallucinated response.

3. **After any re-take**: log the result via `nhl-input --log-step`. The
   `decision` field should be `recover` if the re-take confirms mismatch, or
   `navigate` if the re-take resolves the hallucination and returns to goal.

#### 3b. NAVIGATION inner loop

**CAUTION — daemon input discipline:** Multi-input scripts (e.g.
`scroll("dpad_down", 6, 300)`) are allowed and encouraged for well-mapped
sequences. Every script MUST end with `screenshot()` and the resulting
image MUST be analyzed by `menu-vision` before ANY further inputs are sent.
Never chain two scripts without vision+log between them.

**GATE — vision liveness check:** The `menu-vision` subagent (`subagent_type="menu-vision"`) is
the ONLY way to interpret a screenshot. The parent agent cannot view images.
If a `menu-vision` call fails, returns empty, or returns non-JSON: **HALT
immediately.** Do not press any buttons. The loop is: INPUT → SCREENSHOT →
menu-vision → (parse JSON) → consistency checks → goal-match → log-step →
next decision. **No menu-vision response = no further inputs.**

For navigation within the task, follow this tight loop:

1. **ANCHOR**: Re-read `screenshots/$RUN_ID/goal.json`. Confirm the current
   task's `pre_screen_title`, `pre_screen`, `pre_options`, `post_screen_title`,
   `post_screen`, and `post_options`. Re-open `map.md` Navigation Reference.
   You are anchoring to known coordinates — do not guess from memory.

2. **CHECK**: Run the vision prompt via `menu-vision` with NO task
    context (the same prompt for every vision call). Use the **streamlined**
    prompt variant from §6 for routine navigation between known screens,
    and the **exhaustive** variant for first-launch dialogs, EXPLORE mode,
    or any unfamiliar screen. You MUST receive a valid JSON object. If the
    subagent call fails, returns empty, or returns non-JSON: **HALT.** Do
    not send any inputs. Retry the vision call.

   Run **Pass A** consistency checks (C1–C7 from §3a) on the vision response.
   If Pass A fails: treat as INTERRUPT (step 6). Do NOT attempt goal-matching
   on an internally inconsistent response.

   Run **Pass B** goal-matching against the current task's expected screen.
   If goal-matching fails, you are off course — press `B` to back up,
   re-read `goal.json`, and re-navigate from a known anchor in `map.md`.

3. **PLAN**: Consult the `map.md` Navigation Reference table for the shortest
   button path to the destination. If no entry exists, use the vision result
   to decide the next single d-pad step toward the target option.

   **Think critically about the vision response even when confidence is high.**
   If `screen_title` says "Player Movement" but `all_text` includes "MEDIA HUB"
   or `options` are `["MY HIGHLIGHTS", ...]`, these are contradictory. Flag
   the response as `inconsistent` regardless of confidence level. The vision
   model can hallucinate with high confidence — the structured fields are your
   defense.

4. **INPUT**: Send a script ending with `screenshot()`:

   ```bash
   # Single button:
   ./target/debug/nhl-input --send 'tap("<button>"); wait(<time>); screenshot("step_N");'

   # Multi-button scroll (atomic within daemon — efficient for known counts):
   ./target/debug/nhl-input --send 'scroll("dpad_down", 6, 300); wait(0.5); screenshot("step_N");'

   # Triggers (use tap_trigger — tap("rt") / tap("lt") will NOT work):
   ./target/debug/nhl-input --send 'tap_trigger("rt", 500); wait(0.5); screenshot("step_N");'
   ```

   **IMPORTANT — Trigger discipline:** LT/RT are analog triggers, not digital
   buttons. `tap("rt")` and `tap("lt")` silently do nothing. Use `tap_trigger`
   or the explicit `set_axis` pattern. Valid trigger names: `"lt"`, `"rt"`,
   `"left_trigger"`, `"right_trigger"`.

   **IMPORTANT — scroll discipline:** `scroll(direction, count, delay_ms)`
   does N sequential taps with a 200ms hold per press, pausing `delay_ms`
   between releases. Valid directions: `"dpad_up"` / `"up"`, `"dpad_down"` /
   `"down"`, `"dpad_left"` / `"left"`, `"dpad_right"` / `"right"`.

5. **VERIFY**: Run the vision prompt via `menu-vision` on the new
    screenshot. Same halt rule as CHECK — you MUST receive a parseable JSON
    response before sending any further inputs. Use the streamlined prompt
    for routine navigation, exhaustive for unfamiliar screens.

   Run **Pass A** consistency checks. If they pass, compare against the
   expected intermediate screen from your plan. If the screen doesn't match
   what the planned step should produce, treat as INTERRUPT (step 6).

   Log the result via `nhl-input --log-step`.

6. **INTERRUPT**: If Pass A consistency checks fail, or if Pass B goal-matching
   fails and the screen is clearly not where you expect, **stop sending inputs
   immediately**. Do not press more buttons trying to fix the situation — that
   drifts you further from the anchor.
   Press `B` ONCE to back up one screen. Take a screenshot and verify you
   are at a well-known screen. Re-enter the loop from step 1 (ANCHOR) with
   the current `goal.json` and `map.md`. Do NOT abort the task — INTERRUPT
   is a normal navigation recovery step, not a terminal state.

7. **REPEAT** from step 1 until Pass B goal-matching succeeds against the
   task's `post_screen_title`, confirmed by §3c (POST-CHECK).

#### 3c. POST-CHECK (MANDATORY)

Take a screenshot, run the vision prompt via `menu-vision` (streamlined for
routine, exhaustive for unfamiliar). Log the result.

Run **Pass A** consistency checks (C1–C7 from §3a). If Pass A fails: enter
RECOVERY protocol (§3d). Do NOT skip tiers.

If Pass A passes, run **Pass B** goal-matching against the task's
`post_screen_title` and `post_options`:

1. `vision.screen_title` contains `post_screen_title` (case-insensitive
   substring)
2. `vision.options` have ≥70% overlap with `post_options` (if `post_options`
   is non-empty)
3. `vision.layout` matches the expected layout for the destination screen
   (from `map.md`)

- **If all three pass**: Assessment → `goal_match`.
  1. Log the result via `nhl-input --log-step`.
  2. Mark the task `"status": "completed"` in `goal.json`.

- **If any fails**: Assessment → `goal_mismatch`. Log the discrepancy via
  `nhl-input --log-step`. Then enter the **RECOVERY protocol** (§3d). Do NOT
  skip tiers. Do NOT abort until you have exhausted all tiers.

Update `goal.json` on disk after every status change.

#### 3d. RECOVERY protocol (when lost or match fails)

**YOU ARE NOT ALLOWED TO ABORT A TASK without exhausting all four tiers.**
Each tier must be attempted with screenshot verification before escalating.

**Tier 1 — One-step back**: Press `B`, wait 1.5s, screenshot. Run the vision
prompt via `menu-vision` (use **exhaustive** — you are lost, don't assume
you know the screen). Run Pass A checks. Is this a screen you
recognize from `map.md`? If yes, re-plan from here. If not, go to Tier 2.

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
3. **Verify end state**: Run the unified vision prompt via `menu-vision` with
   the user's intended end state. Run Pass A checks, then compare
   `screen_title` and `options` against the user's intended end state.
   Is the agent at the correct screen?
4. **If yes**: Set `"completion_gate": true` in `goal.json`. Report success
   to the user with evidence (relevant screenshot paths and what they show).
   Only then proceed to cleanup.
5. **If no**: Do NOT clean up. Report the discrepancy to the user and abort.

### Anti-drift rule

The vision subagent returns a pure description every time — there is no
`match` field or cataloging mode distinction. The main agent must always:

1. Run Pass A consistency checks on every vision response.
2. Run Pass B goal-matching against the current task's expected screen.
3. If the response seems contradictory (e.g. `screen_title` says one thing
   but `all_text` or `options` tells a different story), flag as
   `inconsistent` and trigger INTERRUPT, regardless of `confidence` level.

### Run log and diagnostics

The agent MUST maintain a single per-run log file: `screenshots/$RUN_ID/run_log.jsonl`.
This replaces the previous multi-file approach (`vision_log.jsonl`,
`discrepancies.jsonl`, `state.json`). Each line captures one decision cycle:
prompt sent, vision response, assessment, decision, and plan.

**`run_log.jsonl`** — one line per step, written via `nhl-input --log-step`:

```json
{"step":18,"screenshot":"screenshots/.../018_step_018.png",
 "vision_prompt":"Describe this screenshot from an NHL Legacy hockey game...",
 "vision_response":{"all_text":[...],"screen_title":"CUSTOMIZE","layout":"list","options":[...],"confidence":"high"},
 "assessment":"goal_match","decision":"navigate",
 "plan":"Press A to enter ROSTER MANAGEMENT submenu"}
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
