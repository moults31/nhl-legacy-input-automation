---
name: nhl-retro-analyzer
description: >-
  Analyze a completed NHL Legacy run directory to diagnose navigation errors,
  identify stuck loops, detect map.md drift, and produce actionable
  recommendations. Combines merged data analysis (via generate-report.py) with
  targeted screenshot review for flagged hotspots only — avoids loading every
  screenshot into context.
---

# nhl-retro-analyzer

Analyze a completed NHL Legacy run directory. The skill produces a diagnostic
report identifying where the navigator got stuck, made wrong turns, or
encountered unexpected dialogs. It cross-references navigation paths against
`map.md` to detect stale or incorrect map entries.

**Key constraint:** Only view screenshots for *flagged hotspots*. Do NOT load
every screenshot into context. The merge report tells you where to look.

## Workflow

### Step 0: Gather inputs

The user must provide a run directory path:

```
screenshots/<RUN_ID>/
```

The directory must contain at minimum `daemon_events.jsonl` and
`run_log.jsonl`. If `goal.json` exists, the task context will be included.

### Step 1: Run the merge + hotspot detection

```bash
python3 scripts/generate-report.py screenshots/<RUN_ID>
```

This produces:
- `screenshots/<RUN_ID>/report.html` — interactive HTML report for humans
- `screenshots/<RUN_ID>/hotspot_summary.json` — machine-readable hotspot data
- A hotspot summary printed to stdout — read this first

**Read the hotspot summary from stdout.** It tells you the mismatch rate,
recovery count, long pauses, recovery spirals, and wrong-menu entries without
you needing to parse the full merged data.

### Step 2: Load the hotspot summary and goal

Read these two JSON files:

```
screenshots/<RUN_ID>/hotspot_summary.json
screenshots/<RUN_ID>/goal.json           (if it exists)
```

Do NOT read `run_log.jsonl` or `daemon_events.jsonl` directly — use the
summary.

### Step 3: Context-free hotspot triage

Before viewing ANY screenshots, classify each hotspot by severity:

| Severity | Signal |
|----------|--------|
| **Critical** | Recovery spirals (3+ consecutive recover/B-press steps), wrong-menu entries in unrelated submenus (e.g., Creation Zone when you wanted Roster Management), mismatch rate > 50% on a single task |
| **Major** | Long pauses > 60s (agent was confused), 2+ mismatches on the same expected screen, low-confidence vision responses |
| **Minor** | Single mismatches resolved in 1 step, long pauses 30-60s, one-off B-presses that recovered correctly |

### Step 4: Per-hotspot diagnosis (with targeted screenshots)

For each **Critical** and **Major** hotspot, do this (skip Minor unless
time/context permits):

#### 4a. Read the relevant run_log entries

Read just the entries for the flagged steps from
`screenshots/<RUN_ID>/run_log.jsonl`. Use `sed` to extract specific lines:

```bash
sed -n '5,10p' screenshots/<RUN_ID>/run_log.jsonl
```

For steps with no run_log entry (unlogged), use the daemon events:

```bash
sed -n '40,55p' screenshots/<RUN_ID>/daemon_events.jsonl
```

#### 4b. View the screenshot

Use `Read` on the flagged screenshot file. The hotspot summary gives you
the exact screenshot path.

#### 4c. Cross-reference with map.md

For the flagged step:
1. What screen was **expected** (from goal.json `pre_screen` / `post_screen`)?
2. What screen was **actually seen** (from vision_response `screen` / `actual_screen`)?
3. What button path does `map.md` document to reach the expected screen?
4. What buttons were **actually sent** (from daemon_events `command` entries)?

Answer: did the agent *deviate* from the documented path, or is the map
path itself wrong?

#### 4d. Classify the root cause

Choose ONE root cause per hotspot:

| Root cause | How to detect | Fix |
|------------|--------------|-----|
| **Unexpected dialog** | `actual_screen` is "Autosave Information", "Profile Warning", "Tutorial prompt", "Choose Your Favorite Team" | Add dialog handling to the skill's navigation preamble or to map.md hazards |
| **Scroll overshoot** | Vision shows correct menu BUT the wrong item is highlighted (e.g., selected options don't match expected) | Adjust `scroll()` count in map.md Navigation Reference |
| **Screen name ambiguity** | `match: true` but the returned options don't match `pre_options` — two screens share the same name | Update `goal.json` with `pre_options` or make `pre_screen` more specific in map.md |
| **Wrong menu entry** | Agent entered a completely different submenu (e.g., Creation Zone instead of Roster Management) | Wrong button path or scroll count. Update map.md with correct count |
| **Vision model error** | `match: false` but the screen IS correct — vision hallucinated the wrong title | Add `pre_options` to `goal.json` as a second verification factor |
| **Input dropout** | Command was sent but screenshot shows no change (need daemon_events timestamp analysis) | Increase tap duration with `tap_ms` or add wait after input |
| **Map.md missing path** | Navigation Reference has no entry for the attempted destination | Run EXPLORE mode to map the path, then update map.md |
| **Menu wrap-around** | Agent scrolled past the last option and wrapped to the top (or vice versa) | Document the wrap behavior in map.md; prefer upward scroll if close to top |

### Step 5: Produce the analysis report

Write to `screenshots/<RUN_ID>/analysis.md`:

```markdown
# Run Analysis: <RUN_ID>

**Goal:** <from goal.json goal field>
**Date analyzed:** <today>
**Overall:** X mismatches / Y logged steps (Z% mismatch rate)

## Hotspot Summary

| Severity | Count | Description |
|----------|-------|-------------|
| Critical | N | ... |
| Major    | N | ... |
| Minor    | N | ... |

## Detailed Findings

### <Severity>: <One-line title>

- **Steps:** <step numbers>
- **Expected:** <what goal.json expected>
- **Actual:** <what vision saw>
- **Inputs sent:** <brief description of daemon commands>
- **Map.md check:** <does the documented path match what was attempted?>
- **Root cause:** <from the table above>
- **Recommendation:** <concrete fix — update map.md, adjust skill instructions, etc.>
- **Evidence:** <screenshot path>

### ... (repeat for each hotspot)

## Recommendations

1. **<Action>** — update `<file>`: <what to change>
2. ...

## map.md Updates Needed

- [ ] <specific change to map.md>
- [ ] ...

## Unlogged Steps

<COUNT> steps have no run_log entry. This means the per-step logging was
not active for these steps (added mid-run, or agent missed the log-step call).
Review the nhl-menu-navigator skill to ensure logging is enforced at every step.
```

### Step 6: Report findings to the user

Summarize the top 3 findings with specific, actionable fixes. Do NOT dump the
entire analysis — highlight what matters most.

---

## Interpreting Hotspot Patterns

### Recovery spirals

A recovery spiral is 3+ consecutive steps where the decision was "recover"
or the daemon command was a B-press. This means the agent got lost and kept
pressing B to back out, without reaching a known anchor.

**Diagnosis checklist:**
- What screen was the agent on when the spiral started?
- Did it reach a well-known anchor (Main Menu, CUSTOMIZE)?
- If not: why didn't Tier 3 (full reset from Main Menu) trigger?

**Common causes:**
- The vision model kept misidentifying screens on the way back
- The agent was pressing B but the game needed A to dismiss a dialog first
- map.md has no B-to-anchor chain documented for the starting screen

### Long pauses (30s+ between screenshots)

A long pause means the agent spent a lot of time between sending a command and
taking the next screenshot. This is normal for agent reasoning (10-30s), but
pauses > 60s suggest:
- The agent was re-reading map.md or goal.json
- A vision model call failed and was retried
- The agent drafted a complex recovery plan

**Diagnosis:**
- Check the plan text for that step — is it unusually long or hedging?
- Check the vision confidence — was it low, causing the agent to hesitate?

### Wrong menu entries

When `match: false` and the `actual_screen` is a completely unrelated menu
(e.g., in Creation Zone when navigating to Roster Management), the agent
entered the wrong submenu. Common causes:
1. Scroll count in map.md is off by 1
2. The menu has a flyout panel (COMMUNITY) that intercepted input
3. The CUSTOMIZE menu has a different layout than expected

---

## Cross-Referencing map.md

When cross-referencing with `map.md`, always check:

1. **Is the destination in the Navigation Reference table?**
   If not, the agent was navigating without a documented path — that's the
   root cause.

2. **Does the documented path match actual game behavior?**
   Compare the button sequence in map.md against the actual daemon commands.
   If the agent FOLLOWED the map and still ended up wrong, the map entry is
   stale.

3. **Are navigation hazards documented for this screen?**
   Dialogs (Autosave, Profile Warning, Tutorial, Choose Favorite Team) are
   common interrupters. If a dialog appeared and map.md doesn't list it under
   the parent screen's hazards, add it.

4. **Are scroll counts specific?**
   "Navigate to ROSTER MANAGEMENT (↓×6)" — verify the count against actual
   daemon events. If the agent scrolled 6 times but landed on the wrong item,
   the count is wrong. Count the actual options needed.

---

## Context Management

The analysis works WITHOUT loading all screenshots. The workflow:

1. **hotspot_summary.json** (text, ~1KB) → identifies WHERE to look
2. **run_log.jsonl** (targeted sed lines) → shows WHAT the agent was thinking
3. **daemon_events.jsonl** (targeted sed lines) → shows WHAT inputs were sent
4. **Screenshot** (one PNG per hotspot) → visual confirmation

**NEVER:**
- Read `run_log.jsonl` in its entirety (can be 100+ lines; parse with sed)
- Read all PNG files (view only the ones flagged by hotspot summary)
- Read `daemon_events.jsonl` in its entirety (can be 500+ lines)

---

## Example: Analyzing 20260710_194059_run

Given the hotspot summary for this run:

```
Total screenshots: 31
Logged steps:      16
Mismatches:        6 (37.5%)
Recovery spirals:  1
Long pauses:       4
Wrong menu entries:5
```

**Triage:**

1. **Critical — Recovery spiral at steps 18-19:**
   Screenshots: `017_step_018.png`, `018_step_019.png`
   View these two screenshots. Cross-reference with map.md Navigation
   Reference for ROSTER MANAGEMENT. Check if the agent scrolled past
   ROSTER MANAGEMENT into CREATION ZONE and then had to B-back.

2. **Major — Long pause at step 3 (126s):**
   Screenshot: `002_step_003.png`
   This is the "Press Start Screen" mismatch. The agent took 126s to
   decide to press Start. Check the plan text — was it a complex
   multi-step plan?

3. **Major — Wrong menu entries (5 total):**
   4 of the 5 are unexpected dialogs: Autosave Warning, Profile Warning,
   Choose Your Favorite Team, Tutorial prompt. These are handled correctly
   by the agent (assessed as mismatch, navigated through). One is the
   Creation Zone entry (step 18).
   → Recommend adding these dialogs to map.md hazards under "First Launch
   Sequence" so the agent can anticipate them.

---

## References

| Doc | Covers |
|-----|--------|
| `map.md` | Persistent menu graph, Navigation Reference table, hazards |
| `scripts/generate-report.py` | Merge + hotspot detection + HTML report |
| `.agents/skills/nhl-menu-navigator/SKILL.md` | Navigation skill (to check for missing instructions) |
