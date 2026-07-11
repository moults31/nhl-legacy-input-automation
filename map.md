# NHL Legacy Menu Map & Operational Notes

Persistent, cross-run reference. Used by `nhl-menu-navigator` for navigation
planning (EXECUTE mode reads it; EXPLORE mode writes to it). Agents: update
this file with any new screens, transitions, or gotchas you discover. Do not
log journals here — keep it reference-style.

## Menu Graph

```
GAME BOOT
└── Language Selection              → A → Main Menu

TITLE SCREEN ("Press Start")        → Start → (First launch: Autosave dialog)
AUTOSAVE INFORMATION                → A (Okay) → Main Menu

MAIN MENU
├── PLAY                           [A]  → Play submenu
│   ├── PLAY NOW                         → Team Selection
│   ├── HOCKEY ULTIMATE TEAM             (avoided: online-dependent)
│   ├── QUICK MODES                 [A]
│   │   ├── NHL 94 Anniversary Mode      (avoided: complex)
│   │   ├── NHL Moments Live             → Internal mode menu
│   │   ├── Winter Classic               → Venue Selection
│   │   └── Training                [A]
│   │       ├── PRACTICE MODE
│   │       ├── SHOOTOUT MODE
│   │       └── TUTORIALS
│   ├── ONLINE                      [A]
│   │   ├── GM Connected
│   │   ├── Online Versus Play
│   │   ├── EA SPORTS Hockey League
│   │   ├── Online Team Play
│   │   └── Online Shootout
│   ├── CAREER                      [A]
│   │   ├── LIVE THE LIFE                (avoided: complex)
│   │   ├── BE A GM MODE            [A]  → Entry (NEW / LOAD)
│   │   │   ├── NEW                 [A]  → Setup → Team Select → Save → Hub
│   │   │   │   ├── QUICK SETTINGS       (Tab 1/3)
│   │   │   │   ├── RULES                (Tab 2/3)
│   │   │   │   ├── ADVANCED SETTINGS    (Tab 3/3)
│   │   │   │   ├── SELECT TEAM          (LT/RT teams, LB/RB GM skill, Y name)
│   │   │   │   │   └── Salary Cap dialog → Yes / No / Cancel
│   │   │   │   └── SAVE NAME            (default "BE A GM1_")
│   │   │   │       └── BE A GM HUB
│   │   │   │           A = Advance sim, Y = Phone menu, Start = Game Menu
│   │   │   │           LB/RB = team, LT/RT = month, RS = Messages
│   │   │   │           Phone: GM Tracker, Trades, Free Agents, Rosters, etc.
│   │   │   └── LOAD                     → Load existing save
│   │   └── SEASON MODE
│   └── TOURNAMENTS                 [A]
│       ├── PLAYOFF MODE
│       ├── TOURNAMENT MODE
│       └── BATTLE FOR THE CUP
│
├── COMMUNITY                       [A]  (flyout panel — see Navigation Hazards)
│   ├── VIEW PLAYER HUB
│   ├── LEADERBOARDS
│   ├── LOBBY
│   └── MY HIGHLIGHTS
│
├── CUSTOMIZE                       [A]
│   ├── CREATION ZONE               [A]
│   │   ├── CREATE PLAYER
│   │   ├── CREATE TEAM
│   │   ├── EDIT PLAYER
│   │   └── EDIT TEAM
│   ├── CUSTOMIZE AI
│   ├── EA SPORTS MEDIA HUB
│   ├── FAVORITE TEAM
│   ├── OFFER CODE ENTRY
│   ├── PROFILE MANAGEMENT
│   ├── ROSTER MANAGEMENT           [A]
│   │   ├── TEAM ROSTERS
│   │   ├── PLAYER MOVEMENT              → Two-column trade screen
│   │   ├── EDIT LINES
│   │   ├── JERSEY NUMBERS
│   │   ├── SET DEFAULT ROSTERS
│   │   └── DOWNLOAD ROSTERS
│   ├── SAVE/LOAD/DELETE            [A]
│   │   ├── SAVE
│   │   ├── LOAD                    [A]  → File-type selector
│   │   │   ├── ROSTERS                  → Roster file list
│   │   │   ├── BE A GM
│   │   │   ├── TOURNAMENTS
│   │   │   ├── BE A PRO
│   │   │   ├── PLAYOFF MODE
│   │   │   ├── SEASON MODE
│   │   │   └── NHL LEGACY EDITION PROFILE
│   │   └── DELETE
│   └── SETTINGS                    [A]
│       ├── HOSPITALITY SETTINGS
│       ├── USER CELEBRATIONS
│       ├── CONTROLLER SETTINGS
│       ├── VIDEO CALIBRATION
│       ├── RULES
│       ├── GAMEPLAY SETTINGS
│       ├── VOLUME SETTINGS
│       ├── VISUAL SETTINGS
│       ├── ONLINE SETTINGS
│       └── CREDITS
│
├── FEATURED
│   ├── HOCKEY ULTIMATE TEAM             (avoided)
│   ├── LIVE THE LIFE                    (avoided)
│   └── NHL 94 ANNIVERSARY MODE          (avoided)
```

## Navigation Reference

### How to reach a destination

| Destination | Path from Main Menu |
|---|---|
| Team Selection (Play Now) | `A` (PLAY), `A` (PLAY NOW) |
| Quick Modes | `A` (PLAY), `↓ ↓` (QUICK MODES), `A` |
| Training | Quick Modes → `↓ ↓ ↓` (Training), `A` |
| Online | `A` (PLAY), `↓ ↓ ↓` (ONLINE), `A` |
| Career | `A` (PLAY), `↓ ↓ ↓ ↓` (CAREER), `A` |
| Tournaments | `A` (PLAY), `↓ ↓ ↓ ↓ ↓` (TOURNAMENTS), `A` |
| Customize | `↓ ↓` (CUSTOMIZE), `A` |
| Creation Zone | Customize → `A` |
| Roster Management | Customize → `↓` ×6 (ROSTER MANAGEMENT), `A` |
| Player Movement (trade) | Roster Management → `↓` (PLAYER MOVEMENT), `A` |
| Save/Load/Delete | Customize → `↓` ×7 (SAVE/LOAD/DELETE), `A` |
| Load Roster | Save/Load → `↓` (LOAD), `A` → `↓` (ROSTERS), `A` → select file, `A` → confirm Proceed |
| Settings | Customize → `↓` ×8 (SETTINGS), `A` |
| BE A GM MODE | PLAY → `↓` ×4 (CAREER), `A` → `↓` (BE A GM MODE), `A` → `A` (NEW) |
| Be A GM Hub (any phone option) | See be-a-gm-hub section below |

### On-screen button hints

- **A** — Select / Advance
- **B** — Back
- **Y** — Game Manual
- **X** — REDEEM COINS
- **LT / RT** — Page navigation
- **RS** — Scroll Text

### Default focus

On the Main Menu, the cursor defaults to **COMMUNITY** on first
entry. Navigate to other items with d-pad up/down.

## Player Movement (Roster Trades)

The Player Movement screen (`Customize → ROSTER MANAGEMENT → PLAYER MOVEMENT`)
is a **two-column layout** for swapping players between teams.

### Layout

```
┌─────────────────────┬─────────────────────┐
│  Left Panel (team)  │  Right Panel (team) │
│  ← d-pad left       │  d-pad right →      │
│  LT/RT: cycle team  │  LT/RT: cycle team  │
│  LB/RB: cycle league│  LB/RB: cycle league│
│                     │                     │
│  [player list]      │  [player list]      │
│                     │                     │
│  Moving Block       │  Moving Block       │
│  (staged player)    │  (staged player)    │
└─────────────────────┴─────────────────────┘
         X = Execute Move
```

### How to trade

1. **Switch panels**: `dpad_left` / `dpad_right` to highlight the desired
   column. The active panel shows a green indicator on the selected player.
2. **Cycle teams**: `LB` / `RB` to switch between teams within the current
   league. Tampa Bay Lightning (TBL) and Florida Panthers (FLA) are in NHL.
3. **Cycle leagues**: `LT` / `RT` to switch between NHL, SHL, Free Agents,
   etc.
4. **Select player**: `A` on a player stages them in that panel's Moving
   Block.
5. **Switch panels** and repeat to stage a second player from the other
   team.
6. **Execute**: `X` to complete the swap.
7. **Drop to Free Agency**: `Y` instead of `X` sends the staged player to
   the Free Agent pool (one-sided, not a trade).

### Operational notes

- **d-pad left/right is NOT listed in on-screen hints** but is the only way
  to switch between the two panels. Without it you're stuck on one side.
- **LT/RT are analog triggers, not buttons.** `tap("rt")` and `tap("lt")` do
  NOT work — they silently do nothing. Use `tap_trigger("rt", 500)` or the
  explicit `set_axis` pattern:
  ```
  set_axis("right_trigger", 1.0); wait(0.5); set_axis("right_trigger", 0.0); wait(0.4);
  ```
- **LB/RB are digital bumpers.** Use `tap("rb")` or `tap("lb")` — these work.
- When cycling teams with triggers, the game may occasionally jump multiple
  teams if the axis signal is polled during a transition. A hold time of
  500ms and a release wait of 400ms per press has been most reliable.
- The roster file list under `LOAD → ROSTERS` shows saved rosters sorted by
  last-modified date (newest first). The first entry is highlighted by
  default.
- Loading a roster triggers a confirmation dialog ("All unsaved changes will
  be lost"). Navigate to **Proceed** (`↑` then `A`) to confirm.
- After loading, the game returns to the Roster file list; press `B` to
  back out to the Customize menu.
- When entering Player Movement, the left panel defaults to the first NHL
  team (Anaheim Ducks) and the right panel defaults to Free Agents. Both
  panels can be independently set to any league/team.

## BE A GM MODE

### Entry Flow

```
BE A GM MODE (NEW / LOAD)  → A (NEW)
  → Quick Settings (Tab 1, LT/RT for tabs)
  → Rules (Tab 2)
  → Advanced Settings (Tab 3)
  → A (Advance) through all tabs with defaults
  → SELECT TEAM screen
    → LT/RT: cycle teams, LB/RB: GM skill level
    → A (Advance)
    → [Salary Cap dialog if over cap. Cancel is highlighted by default.
     ↑×2 → "Yes – CPU will automatically fix teams", A to confirm.
     WARNING: pressing A on Cancel returns to team selection — do not skip this dialog.]
  → Save Name keyboard (default "BE A GM1_")
    → From the "a" key: ↓×2 → Space, →×1 → Done, A to confirm
  → Loading → BE A GM HUB

#### Team Selection details

On the SELECT TEAM screen each team shows:
- **OFF/DEF/GOA star ratings** (0.5–5★)
- **AHL Affiliate** (e.g., Chicago Wolves) with OVR rating
- **Players Under Contract** count, **Average Age**, **Top Player** name
- **Salary Cap Available** (+/–$), **Cap Value**
- **Staff bars**: Amateur Scout, Pro Scout, Medical Staff, Asst. Coach
- **TP for Legend**: total training points to reach Legend rank (4,800 at default)

Button hints: `A Advance`, `B Help`, `X Substitute Team`, `Y Edit GM First Name`, `LT/RT Cycle teams`, `LB/RB GM Skill Level`.
```

### Setup Tabs Detail

**Quick Settings (Tab 1):**
Skill Level (Pro), Game Style (Simulation), Period Length (5 Min),
Be A GM Length (25), Season Starting Date (Regular Season), CPU Trades (On)

**Rules (Tab 2):**
Period Length, Offsides (Delayed), Icing (Hybrid), Penalties (On),
Post Whistle Rules (Relaxed), Penalty Time Scaling

**Advanced Settings (Tab 3):**
Tuner Set Version (Latest), Playoff Series Length (Best of 7),
Playoff Tie Break (Continuous OT), Season Overtime Loss (1 Pt),
Season Tie Break (5min 4v4 → Shootout), Asst Coach Edits Lines (Yes)

### Hub Layout

Weekly calendar (Sun–Sat) with Home/Away indicators, Team Leaders panel,
Injuries panel, Message Center.

| Button | Action |
|--------|--------|
| **A** | Advance (sim time / open GM Tracker) |
| **Y** | Expand → Phone "Go To..." menu |
| **Start** | Game Menu (Continue / Main Menu / Save Game) |
| **X** | Use Phone (same as Y?) |
| **B** | Help |
| **LB/RB** | Switch team |
| **LT/RT** | Switch month |
| **RS** | Message Center |

**Phone menu back behavior:** Pressing `B` from a phone sub-screen (e.g., Free Agents, Team Rosters) closes **both** the sub-screen AND the phone menu, returning to the bare hub. Press `Y` again to reopen the phone menu.

**Attract mode:** The hub enters an attract/demo state after ~30s idle — gameplay animates behind the UI. This is normal. Do NOT treat it as a navigation error; any d-pad input returns focus to menus.

### Phone "Go To..." Menu

| # | Option | Description seen on hover |
|---|--------|--------------------------|
| 1 | **GM Tracker** | "View info on how you are doing as GM." |
| 2 | **Trade Players** | Two-column trade: 6 slots/side, LT/RT teams, salary cap. Shows retained cap, cap total per side, league approval status, and GM relationship. |
| 3 | **Trading Block** | List of trade-offer players |
| 4 | **Free Agents** | Player table + salary cap summary + offer contracts. LT/RT filter by All Skaters tab; LB/RB toggle between All and Skaters. Sort by OVR, POS, ROLE, AGE, Salary. |
| 5 | **Scout Assignment** | Scout management |
| 6 | **Contracts** | Contract review |
| 7 | **Team Rosters** | Roster editing |
| 8 | **Transaction News** | League trade log |
| 9 | **Staff Upgrades** | Hire/upgrade staff |

GM Tracker shows: GM Reputation (Level 1), TP (11/11), Roster Grades,
Easiest Tasks (objectives with TP rewards — e.g., "Win a game by at least
3 Goals" for 2 TP, "Play one Game" for 3 TP, "Sign one Free Agent" for 3 TP),
Amateur GM Award (Progress 100%). Tabs: Franchise Makeup, Franchise Analysis.

First-launch hazards (Autosave, Profile, Favorite Team, Tutorial) may appear
between Title Screen and Main Menu. See First Launch Hazards section.

## First Launch Hazards

After pressing Start on the title screen, zero or more of these dialogs
may appear. They can come in **ANY ORDER** and some may not appear at all
depending on whether a profile exists and whether this is the first boot.
Treat each dialog as a normal screen transition — dismiss, verify with
vision, then continue.

| Screen | Recognition | Dismissal |
|--------|-------------|-----------|
| Autosave Information | Dialog box: "Autosave Information", single "Okay" button | `A` |
| Profile Warning Screen | Options: "Sign In", "Retry", "Continue Without Saving" | `↓` to "Continue Without Saving", `A` |
| Choose Your Favorite Team | Grid of NHL team logos with team names | Select any team with d-pad, `A` |
| Tutorial prompt | Options: "Yes", "No" — asks "Would you like to enter Tutorial Mode?" | `↓` to "No", `A` |

The full possible chain is:
```
Title Screen (Start) → [Autosave?] → [Profile Warning?] → [Choose Favorite Team?] → [Tutorial?] → Main Menu
```

DO NOT assume a fixed order or that all dialogs appear. After each `A`-press
to dismiss, take a screenshot and verify with vision before sending the next
input.

## Navigation Hazards

### COMMUNITY flyout

COMMUNITY is a **flyout panel**, not a full-screen menu. When highlighted
and `A` is pressed, its submenu expands in-place and **intercepts d-pad
navigation** — pressing `↓` from COMMUNITY enters the submenu instead of
moving to CUSTOMIZE. To reach CUSTOMIZE, close the flyout first with `B`.

- **Open**: `A` on COMMUNITY
- **Close**: `B` (single tap)
- **Auto-close**: After ~15–30s of idle, the flyout retracts
- **To reach CUSTOMIZE**: `B` to close flyout, then `↓`

### Menu wrapping

All vertical menus wrap around — scrolling past the last item returns to
the first, and vice versa.

### Confirmation on back

Pressing `B` from the Team Selection screen triggers a Yes/No confirmation
dialog. To actually go back: `↑` (select Yes), `A`.

## Input Timing

| Action | Wait after input |
|---|---|
| Menu item scroll (d-pad) | 0.5s |
| Screen transition (A to enter) | 2.0–3.0s |
| Screen transition (B to go back) | 1.5–2.0s |
| Game startup / title screens | up to 10s |

### Tap duration

Default `tap()` holds for 200ms (sufficient at 60fps). At low FPS (e.g.,
10fps cap), use `tap_ms("btn", 300)` or `hold("btn", 0.3)` if inputs are
dropped.

For multi-press navigation, use `scroll("dpad_down", 6, 300)` instead of
shell `for` loops. `scroll` runs entirely inside the daemon, avoiding the
parallel `--send` timeout issue. Each press holds 200ms; `delay_ms` is the
pause between releases.

**Triggers at low FPS:** Under a 10fps cap, `tap("rt")` or `tap("lt")` can
be dropped. Use explicit axis setting with a hold duration instead:

```rhai
set_axis("right_trigger", 1.0);
wait(0.3);
set_axis("right_trigger", 0.0);
wait(0.8);
```

## Operational Gotchas

- **Title screen**: Requires Start to advance, not A. Other screens accept A.
- **Game process tree**: Three processes (python3 proton launcher →
  steam.exe → nhllegacy.exe). Kill all three to fully stop the game. Use
  `scripts/kill-nhl.sh`.
- **Game window substring**: `"nhllegacy"` matches the Proton window. Title
  contains "rexglue-v..." version string.
- **Triggers (LT/RT)**: Analog axes. `tap("rt")` / `tap("lt")` silently do
  nothing. Use `tap_trigger("rt", 500)` or `set_axis("right_trigger", 1.0)`.
  See Player Movement Operational Notes for detailed trigger usage.
- **Run ID validation bug**: `validate_run_id_neutral` enforces `len() == 19`,
  so only `_run` suffix (19 chars) is accepted. `_explore` (23 chars) is
  rejected. Tracked in `crates/app/src/main.rs:662`.

## Avoided Modes

| Mode | Reason |
|---|---|
| HOCKEY ULTIMATE TEAM | Complex setup, online-dependent |
| LIVE THE LIFE | Complex setup (Be A Pro variant) |
| NHL 94 ANNIVERSARY MODE | Complex setup |
| On-ice gameplay | Resource-intensive, derails menu mapping |

## Unexplored

- COMMUNITY sub-submenus (VIEW PLAYER HUB, LEADERBOARDS, LOBBY, MY HIGHLIGHTS)
- CUSTOMIZE leaf nodes: CUSTOMIZE AI, FAVORITE TEAM, OFFER CODE ENTRY, PROFILE MANAGEMENT, EA SPORTS MEDIA HUB
- SETTINGS sub-submenus
- NHL Moments Live internal menu
- Winter Classic — venue selection only; gameplay not explored
- ROSTER MANAGEMENT leaf nodes: TEAM ROSTERS, EDIT LINES, JERSEY NUMBERS, SET DEFAULT ROSTERS, DOWNLOAD ROSTERS
- SAVE/LOAD/DELETE leaf nodes: SAVE, DELETE (only LOAD explored)
- BE A GM sub-screens explored: GM Tracker, Trade Players, Free Agents.
  Still unexplored: Trading Block, Scout Assignment, Contracts, Team Rosters,
  Transaction News, Staff Upgrades.
- BE A GM: Sim gameplay (Advance on hub) — not tested beyond GM Tracker opening
