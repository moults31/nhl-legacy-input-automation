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
│   │   └── SEASON MODE                     [A] → Entry (NEW / LOAD)
│       ├── NEW                        [A] → Setup → Team Select → Save → Hub
│       │   ├── QUICK SETTINGS               (Tab 1/3, LT/RT for tabs)
│       │   ├── RULES                        (Tab 2/3)
│       │   └── ADVANCED SETTINGS            (Tab 3/3)
│       │   ├── SELECT TEAM                  (← to toggle user control, LT/RT divisions)
│       │   └── SAVE NAME                    (default "SEASON1_")
│       └── LOAD                             → Load existing save
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
│   ├── PROFILE MANAGEMENT          [*]
│   │   * Table layout: 3 columns (Profile, Lead, Activated). Rows for
│   │     every detected controller: virtual uinput ("User") + any physical
│   │     Xbox 360 controllers ("Xbox 360 Controller N").
│   │   * "Profile Options" section at bottom with "Activate Profile".
│   │   * **HAZARD (hard blocker):** "Activate Profile" is INERT when
│   │     physical controllers are connected alongside the virtual uinput
│   │     controller. There is no Xbox Live sign-in backend in the recomp
│   │     environment to complete activation.
│   │   * **HAZARD (Guide button):** The Guide button opens the recomp
│   │     overlay (Performance/Display/Rendering), NOT the Xbox profile
│   │     sign-in menu. Profile activation via Guide is impossible through
│   │     uinput.
│   │   * **Resolution:** Disconnect all physical controllers and relaunch
│   │     the game. Only the virtual uinput controller can be active.
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

### Startup Paths (from Game Boot)

| Destination | Path from Title Screen |
|---|---|
| Main Menu | Start, [Autosave: A], [Profile Warning: ↓ (Continue Without Saving), A], [Favorite Team: any, A], [Tutorial: ↓ (No), A] |
| Season Mode Hub | Main Menu → ↑, A (PLAY), ↓×4, A (CAREER), ↓×2, A (SEASON MODE), ↓, A (LOAD), ↓, A (select save) |
| Season Mode Hub (from Main Menu) | ↑, A (PLAY), ↓×4, A (CAREER), ↓×2, A (SEASON MODE), ↓, A (LOAD), ↓, A (select save) |

**Note:** The first-launch dialogs (Autosave, Profile Warning, Favorite Team, Tutorial) may appear in any order or not at all between the Title Screen and Main Menu. See First Launch Hazards section for dismissal instructions.

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
| SEASON MODE | PLAY → `↓` ×4 (CAREER), `A` → `↓` ×2 (SEASON MODE), `A` → `A` (NEW) |
| Season Mode Hub | Season Mode → selection → save → hub (see Season Mode section) |
| Season Mode Trade | Hub → GM OPTIONS → `↓` (TRADE PLAYERS), `A`. Left column locked to user team. |
| Season Mode Save & Exit | Hub → `↓` ×8 (QUIT SEASON MODE), `A` → wait for dialog → `↑` (Save and Exit), `A` |
| Season Mode Save (only) | Hub → `↓` ×7 (CUSTOMIZE), `A` → `↓` ×2 (SAVE SEASON), `A` → select file, `A` → QWERTY keyboard → Done |

### Season Mode Central Hub Navigation

The Season Mode hub has **two views**. The menu view shows 8 items in a left panel;
the calendar view (activated by pressing B from the menu view) fills the screen.

| Input | Menu View | Calendar View |
|-------|-----------|---------------|
| **A** | Select menu item | Play/Simulate on game date |
| **A (non-game date)** | — | Go to Team Standings |
| **B** | Enter calendar-focused view | Move to previous date |
| **Start** / **D-pad right** | Game Menu (CUSTOMIZE overlay) | Game Menu (CUSTOMIZE overlay) |
| **Y** | Expand info panel | Expand info panel |
| **LT/RT** | Cycle month in calendar strip | Cycle month in calendar |
| **LB/RB** | (same as LT/RT) | (same as LT/RT) |
| **RS** | Message Center | Message Center |
| **D-pad left** | — | Help |

### Season Mode Setup Tabs

**Quick Settings (Tab 1):**
League Type (NHL™), Skill Level (Pro), Game Style (Simulation),
Period Length (5 Min), Roster Control (Standard), CPU Trades (On)

**Rules (Tab 2):**
Period Length, Offsides (Delayed), Icing (Hybrid), Injuries (On),
Penalties (On), Post Whistle Rules (Relaxed)

**Advanced Settings (Tab 3):**
Tuner Set Version (Latest), Playoff Series Length (Authentic),
Playoff Tie Break (Continuous OT), Season Overtime Loss (1 Pt),
Season Tie Break (5min 4v4 → Shootout), Asst Coach Edits Lines (Yes)

### Season Mode Central Hub Menu

| # | Item | Type | Sub-options |
|---|------|------|-------------|
| 1 | PLAY NEXT GAME | Action | Opens calendar at next game date |
| 2 | CALENDAR | Action | Opens monthly calendar |
| 3 | GM OPTIONS | Flyout | FREE AGENTS, TRADE PLAYERS, INJURY REPORT, TRANSACTION NEWS, ROSTER MOVES |
| 4 | COACHING OPTIONS | Flyout | PRACTICE MODE, EDIT LINES, VIEW LINES, JERSEY NUMBERS, STRATEGY, TEAM REPORTS |
| 5 | STATS CENTRAL | Flyout | TEAM STANDINGS, TEAM STATS (SEASON/PLAYOFF), PLAYER STATS (SEASON/PLAYOFF), PLAYOFF TREE, AWARDS |
| 6 | EA SPORTS MEDIA HUB | ? | Not explored |
| 7 | CUSTOMIZE | Flyout | EDIT PLAYER, SETTINGS, SAVE SEASON |
| 8 | QUIT SEASON MODE | Action | Confirmation: Yes / Save and Exit / No |

### Sim Game Flow

1. Navigate calendar to a game date (opponent logo shown on cell)
2. Press A → "Simulate up to this day" / Cancel dialog
3. Select "Simulate up to this day" and press A
4. Game processes and hub updates: record, scores, team leaders, messages

**HAZARD:** Cancel is the **default selection** on the sim dialog. The
agent must scroll to "Simulate up to this day" before pressing A, or the
dialog will be dismissed without simming.

After simming, the calendar shows scores on completed game dates (e.g.,
"L 3-0"), the team record updates, the Team Leaders panel populates with
player stats, and the Message Center populates with league news.

### Season Mode Trade Screen vs. Player Movement

The Season Mode trade screen (`Hub → GM OPTIONS → TRADE PLAYERS`) is fundamentally
different from the CUSTOMIZE Player Movement screen. Do not assume the same
controls apply.

| Feature | Player Movement (CUSTOMIZE) | Trade Screen (Season Mode Hub) |
|---------|---------------------------|-------------------------------|
| Left column team | Cycle with LT/RT | **Locked** — user-controlled team |
| Right column team | Cycle with LT/RT | Access with d-pad right, cycle with LT/RT |
| Default rows | Shows a page of players immediately | Empty placeholders (dashes) — press A to open |
| Slots per side | 6 | 5 |
| Execute | X | **X (confirmed)**. On-screen hint says "Y Execute Trade" but Y does nothing — X is the actual execute button. |

**Why the left column is locked:** In Season Mode, you are the user-controlled
team. The left column represents your team and cannot be cycled. If multiple
teams are user-controlled, they are selected from the main Season Mode menu,
not in the trade screen.

**Player selection flow (per slot):**
1. Press `A` on an empty placeholder row → opens player list
2. D-pad up/down to scroll players
3. Press `A` to stage the selected player in that slot

**Team cycling (right column only):**
1. Press d-pad right to activate the right column
2. Use LT/RT triggers to cycle the CPU-controlled team

**Player list controls:**
- `A` select player, `B` back
- `LB/RB` filter: All Skaters / Forwards / Defense / Goalies
- `LS` sort, `RS` player info
- Columns in player list: POS, PLAYER, TRADE VALUE (bar), OVR, AGE, SAL

**HAZARD:** Attempting LT/RT on the left column does nothing because the
user team is locked. This differs from Player Movement where both columns are
freely cyclable. Agents mistaking the trade screen for Player Movement will
waste cycles trying to trigger-cycle the left team.

#### Trade Acceptance / Rejection Factors

CPU-controlled teams evaluate trade offers and can accept or reject them.
A rejection dialog appears with a GM message and an "OK" button to dismiss.
An acceptance shows a confirmation dialog also requiring "OK" to confirm.

**Experiment results (Calgary user team vs Anaheim CPU):**

| Offer (from CGY) | Ask (from ANA) | Result | Likely factor |
|---|---|---|---|
| C. RESCHNY (C, 70 OVR, $0.575M, AHL) | C. KREIDER (LW, 90 OVR, $4.86M) | **Rejected** | Insulting — CPU: "almost insulting. offer up a lot more" |
| M. MCTAVISH (C, 82 OVR, $5.235M) | L. CARLSSON (C, 83 OVR, $0.71M) | **Rejected** | Salary imbalance — CPU: "won't start making bad deals now. offer me way more" |
| J. HUBERDEAU (LW, 86 OVR, $7.85M) | L. BOELIUS (D, 67 OVR, $0.575M) | **Accepted** | CPU wins big — massive upgrade for cheap prospect |

**Observed factors affecting acceptance likelihood:**

- **Salary cap value is critical.** Even when OVR ratings are close (82 vs 83),
  a large salary disparity ($5.235M vs $0.71M) triggered rejection. The CPU
  weighs the financial impact heavily.
- **OVR gap alone isn't enough.** A 1-point OVR difference with massive salary
  gap was rejected. The CPU demands clear surplus value.
- **CPU must perceive a win.** Acceptances occurred when the CPU received a
  dramatically better player (86 OVR) in exchange for a throwaway prospect
  (67 OVR). The CPU evaluates total value — OVR, salary, age, potential — and
  requires a net benefit.
- **Trade value bar** on the player list (a visual bar in the TRADE VALUE
  column) may represent the game's internal valuation. Comparing bars between
  offered and requested players provides a visual rough estimate.
- Once a player is staged in a slot, they are **removed from the other side's
  player list** — you cannot trade for a player already offered.

### Season Mode Team Selection

Unlike BE A GM (highlight = selected), Season Mode requires pressing **←
(dpad_left)** to toggle a team as user-controlled. The counter shows
"User Teams X/30". At least 1 user team required to begin.

**HAZARD:** The on-screen button bar only shows `Help`, `X Substitute Team`,
`A Begin Mode`, and `B Back`. It does NOT show the user-control toggle.
Look for ◁ ▷ arrows flanking the highlighted team row in the team list
table — those indicate that **d-pad left/right** toggles user control. If
you press A (Begin Mode) without a user team assigned, a blocking dialog
appears ("You must have at least one user controlled team to continue").

### Game Menu (Start in Season Mode)

**Note:** Pressing Start from the hub toggles between menu view and calendar view
(not a separate Game Menu overlay). The CUSTOMIZE flyout (open with A on CUSTOMIZE
hub option) provides EDIT PLAYER, SETTINGS, and SAVE SEASON.

During actual gameplay (ice rink), pressing Start opens the in-game pause menu which includes Quit options.

### Season Mode Save/Exit Flows

There are two distinct paths to save and exit Season Mode:

#### Path 1: QUIT SEASON MODE → Save and Exit

From the hub menu view: navigate to `QUIT SEASON MODE` (option 8), press A.
This opens a confirmation dialog with three options:

1. **Yes** — Exit without saving
2. **Save and Exit** — Save progress and exit to main menu
3. **No** — Cancel (highlighted by default)

**NOTE:** The dialog appears over the **calendar view** (not menu view). To
select "Save and Exit", press `↑` once from the default "No" selection.

**HAZARD:** Pressing A on QUIT SEASON MODE from the hub opens a loading screen
(~5s), then the confirmation dialog appears over the calendar view. The dialog
may be partially obscured by other hub panels.

#### Path 2: CUSTOMIZE → SAVE SEASON

From the hub menu view: navigate to `CUSTOMIZE` (option 7), press A to open the
CUSTOMIZE flyout, then scroll to `SAVE SEASON` (option 3), press A.

This opens a save file browser showing existing saves (e.g., SEASON1 through
SEASON5) plus a "Create new Season" option. Selecting an existing save opens a
**QWERTY keyboard** with the save name pre-filled (e.g., "SEASON1_"). Press
**Done** to confirm overwrite.

Keyboard controls: `A` select key, `B` cancel, `Y` Caps, `LB` Shift, `RT` Space,
`LT` special keys, `RS` clear, `X` back, `Done` button on right side.

**NOTE:** This only saves, it does NOT exit. You must still use QUIT SEASON MODE
or navigate the menu hierarchy to leave Season Mode.

#### How to Return to Main Menu from Season Mode

- **Without saving:** QUIT SEASON MODE → Yes
- **With saving:** QUIT SEASON MODE → Save and Exit
- B does NOT exit the hub — in the hub it toggles between menu view and calendar
  view. To back out of Season Mode entirely, use QUIT SEASON MODE.

### SEASON MODE Operational Notes

- **Menu wrapping:** The hub's 8-item menu wraps — scrolling past
  QUIT SEASON MODE goes to PLAY NEXT GAME and vice versa.
- **No dedicated Advance button:** Unlike BE A GM (where A advances time
  from the hub), Season Mode requires explicitly selecting a game date on
  the calendar and choosing "Simulate up to this day" from the dialog.
- **B toggles hub view:** Pressing B from the menu view enters the
  calendar-focused view (no left menu panel). B does NOT exit the hub.
- **Start toggles hub view:** Pressing Start from the hub toggles between
  menu view and calendar view. It does NOT open a separate Game Menu overlay.
- **Unlabeled input:** A on a non-game date in the calendar goes to Team
  Standings. D-pad left in the calendar view opens Help.
- **Message Center:** Populates automatically with league news (trades,
  game results, signing announcements) presented as social-media posts.
- **Trade screen default teams:** The trade screen left column is locked
  to the user-controlled team. The right column defaults to a random CPU team.
  Cycle the right column only with LT/RT after pressing d-pad right.
- **Loading screens:** QUIT SEASON MODE triggers a loading screen (~5s)
  before the confirmation dialog appears. The dialog appears over the
  calendar view, not the menu view.
- **Save file browser:** Both LOAD and SAVE show save files sorted by
  last-modified date (newest first). The SAVE screen offers "Create new
  Season" at the top of the list. Selecting an existing save opens a
  QWERTY keyboard for name editing before confirming the save.

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
| **Profile Activation Required** | Dialog: "You need to activate a lead NHL Legacy Edition profile to be able to save or load your content" | **HALT — see PROFILE MANAGEMENT section below. This is NOT resolvable through navigation. Physical controller interference.** |

The full possible chain is:
```
Title Screen (Start) → [Autosave?] → [Profile Warning?] → [Choose Favorite Team?] → [Tutorial?] → Main Menu
```

DO NOT assume a fixed order or that all dialogs appear. After each `A`-press
to dismiss, take a screenshot and verify with vision before sending the next
input.

**NOTE:** The "Profile Activation Required" dialog does NOT appear during
the first-launch sequence. It appears when attempting to LOAD or SAVE from
any game mode entry screen (SEASON MODE, BE A GM, etc.) without an active
lead profile. See the dedicated section below for handling this blocker.

## Profile Activation Required (in-game hard blocker)

This dialog appears when attempting to LOAD or SAVE from any game mode
entry screen (SEASON MODE, BE A GM, etc.) when no controller has an active
lead profile.

| Field | Value |
|-------|-------|
| Recognition | OCR text: _"You need to activate a lead NHL Legacy Edition profile to be able to save or load your content. Please go to the NHL Legacy Edition Profile Management screen to activate a lead NHL Legacy Edition profile. You cannot do this inside of a game mode."_ |
| Screen | Modal dialog with single "Okay" button (`A` to dismiss) |
| Cause | One or more physical Xbox 360 controllers connected alongside the virtual uinput controller. The game enumerates all controllers but cannot activate a profile for any of them because there is no Xbox Live backend in the recomp environment. |
| Verdict | **HARD BLOCK** — not resolvable through navigation. |

**Detection procedure (OCR):**
```
Does all_text contain "activate a lead NHL Legacy Edition profile"?
→ YES → navigate ONCE to PROFILE MANAGEMENT (CUSTOMIZE → ↓×5, A).
  → OCR for "Xbox 360 Controller" in rows beyond "User".
    → If found: HALT. Physical controllers detected.
    → If NOT found (only "User"): try pressing Y or A on "User" row.
      If Activate Profile fails after 2 attempts: HALT.
→ NO → continue normal flow.
```

**Resolution:**
1. Disconnect ALL physical Xbox 360 controllers from the system.
2. Kill the game process (`scripts/kill-nhl.sh`).
3. Re-launch (only virtual uinput controller active).
4. The game will auto-activate the virtual controller as lead profile.
5. Save/load operations now work.

**Why the Guide button doesn't help:** On a real Xbox 360, pressing the
Guide button opens the Xbox profile sign-in dashboard. In the recomp
environment, the Guide button opens the recomp overlay (Performance,
Display, Rendering, Upscaling settings). Profile activation through the
Guide is impossible via uinput.

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

### CUSTOMIZE flyout (Season Mode Hub)

The CUSTOMIZE flyout in the Season Mode hub behaves identically to the
COMMUNITY flyout on the Main Menu: when open, it **intercepts all d-pad
navigation**. Attempting to navigate from CUSTOMIZE to QUIT SEASON MODE
with `↓` will scroll within the flyout submenu instead.

- **Detection**: OCR or vision response will show both hub items
  (e.g., "GM OPTIONS", "CUSTOMIZE", "QUIT SEASON MODE") AND flyout sub-options
  (e.g., "EDIT PLAYER", "SETTINGS", "SAVE SEASON") simultaneously. If you see
  both, the flyout is OPEN.
- **Close**: `B` (single tap). Verify closure by re-screenshotting — the
  flyout sub-options should disappear.
- **Persistence hazard**: If the CUSTOMIZE flyout is open when QUIT SEASON
  MODE is pressed, it **reappears after the loading screen**. The confirmation
  dialog may be partially obscured, and B-pressing from the dialog can return
  focus to the open flyout instead of the hub. Always B-close the flyout
  BEFORE pressing A on any other hub item.
- **At the hub, check flyout state**: Before any d-pad navigation from the
  hub, verify the flyout is closed. If OCR/vision detects EDIT PLAYER or
  SETTINGS text, press B and re-screenshot.

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
| QUIT SEASON MODE dialog | 5.0s (loading screen before dialog) |
| Trade execution (X) | 4.0–5.0s (dialog appears) |

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

## OCR Engine

Switched from `ocrs` (neural network, models baked at build time) to **tesseract** (C library, system-installed) in
`crates/observer`. Tesseract is dramatically more reliable on game UI text.

- **System deps**: `sudo apt install libtesseract-dev tesseract-ocr-eng`
- **Crate**: `tesseract = "0.15"` — high-level Rust bindings. Uses `tesseract-plumbing` and `tesseract-sys`.
- **Page segmentation mode**: `PSM_AUTO` (3). Works well for single and two-column menu layouts.
- **Output**: Word-level bounding boxes via `TessBaseApi::get_tsv_text(0)`, parsed into `OcrLine`/`OcrWord`.
- **Performance**: ~2-5s per screenshot (debug build). Native C code, faster than ocrs.
- **Selected index detection**: `find_selected_by_luminance` still used to pick the brightest line.
  Works best when the highlighted menu item has higher luminance than background.
- **Accuracy on NHL Legacy menus**: Highly reliable. Extracts menu items ("PLAY", "CUSTOMIZE", "LOAD"),
  save file names ("SEASON1", "AUTOSAVE2"), timestamps, breadcrumbs, and bottom bar hints.
  Some background/stylized text noise is normal.
- **Sidecar auto-creation**: `--ocr` auto-writes `<screenshot>.ocr.json` with provenance for `--log-step`.

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
- **Run ID validation**: `validate_run_id_neutral` accepts both `_run` (19 chars)
  and `_explore` (24 chars) suffixes. See `crates/app/src/main.rs:805`.
- **Physical controller interference**: Physical Xbox 360 controllers
  connected to the system are enumerated by the game as distinct controller
  slots (Xbox 360 Controller 2, 3, 4). They occupy rows in the PROFILE
  MANAGEMENT table with no active profile, making profile activation
  impossible. **Workaround:** disconnect all physical controllers before
  launching the game. If encountered mid-session, HALT and restart.
- **Guide button in recomp**: Pressing the Guide button (aliases: `"guide"`,
  `"xbox"`) opens the recomp rexglue overlay (Performance, Display, Rendering,
  Upscaling & Sharpening), NOT the Xbox 360 profile sign-in menu. This makes
  profile activation via the Guide button impossible through uinput.

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
- SEASON MODE: Sim gameplay (Play/Simulate on calendar) — tested and working
- SEASON MODE: ~~Trade Players screen layout explored~~ → **Now explored: full trade execution,
  acceptance/rejection mechanics, factors, and button mappings documented.**
- SEASON MODE: ~~sub-screens: EDIT PLAYER (Game Menu), EA SPORTS MEDIA HUB~~ → **Now explored: EDIT PLAYER
  screen reached (shows Calgary roster list with player portraits), QUIT SEASON MODE flow mapped,
  SAVE SEASON flow (with QWERTY keyboard) mapped.**
- BE A GM sub-screens explored: GM Tracker, Trade Players, Free Agents.
  Still unexplored: Trading Block, Scout Assignment, Contracts, Team Rosters,
  Transaction News, Staff Upgrades.
- BE A GM: Sim gameplay (Advance on hub) — not tested beyond GM Tracker opening
