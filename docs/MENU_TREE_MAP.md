# NHL Legacy Menu Tree Map

Reference for agents navigating the in-game menu system.

```
MAIN MENU
├── PLAY                     [►]
│   ├── PLAY NOW                 → Team Selection screen
│   ├── HOCKEY ULTIMATE TEAM     (avoided: complex, online-dependent)
│   ├── QUICK MODES              [►]
│   │   ├── NHL™ 94 Anniversary Mode   (avoided: complex)
│   │   ├── NHL™ Moments Live          → Internal mode menu
│   │   ├── Winter Classic             → Venue Selection
│   │   └── Training
│   │       ├── PRACTICE MODE
│   │       ├── SHOOTOUT MODE
│   │       └── TUTORIALS
│   ├── ONLINE
│   │   ├── GM Connected
│   │   ├── Online Versus Play
│   │   ├── EA SPORTS™ Hockey League
│   │   ├── Online Team Play
│   │   └── Online Shootout
│   ├── CAREER
│   │   ├── LIVE THE LIFE              (avoided: complex)
│   │   ├── BE A GM MODE
│   │   └── SEASON MODE
│   └── TOURNAMENTS
│       ├── PLAYOFF MODE
│       ├── TOURNAMENT MODE
│       └── BATTLE FOR THE CUP
│
├── COMMUNITY                [►]  (flyout panel, see §Navigation Hazards)
│   ├── VIEW PLAYER HUB
│   ├── LEADERBOARDS
│   ├── LOBBY
│   └── MY HIGHLIGHTS
│
├── CUSTOMIZE
│   ├── CREATION ZONE
│   │   ├── CREATE PLAYER
│   │   ├── CREATE TEAM
│   │   ├── EDIT PLAYER
│   │   └── EDIT TEAM
│   ├── CUSTOMIZE AI
│   ├── EA SPORTS™ MEDIA HUB
│   ├── FAVORITE TEAM
│   ├── OFFER CODE ENTRY
│   ├── PROFILE MANAGEMENT
│   ├── ROSTER MANAGEMENT              [►]
│   │   ├── TEAM ROSTERS
│   │   ├── PLAYER MOVEMENT            → Two-column trade screen
│   │   ├── EDIT LINES
│   │   ├── JERSEY NUMBERS
│   │   ├── SET DEFAULT ROSTERS
│   │   └── DOWNLOAD ROSTERS
│   ├── SAVE/LOAD/DELETE              [►]
│   │   ├── SAVE
│   │   ├── LOAD                       → File-type selector
│   │   │   ├── ROSTERS                → Roster file list
│   │   │   ├── BE A GM
│   │   │   ├── TOURNAMENTS
│   │   │   ├── BE A PRO
│   │   │   ├── PLAYOFF MODE
│   │   │   ├── SEASON MODE
│   │   │   └── NHL™ LEGACY EDITION PROFILE
│   │   └── DELETE
│   └── SETTINGS
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
├── [FEATURED] ──────────────────────
│   ├── HOCKEY ULTIMATE TEAM         (avoided)
│   ├── LIVE THE LIFE                (avoided)
│   └── NHL™ 94 ANNIVERSARY MODE     (avoided)
```

## Navigation Reference

### How to reach a destination

| Destination | Path from Main Menu (after Start → Autosave dismiss) |
|---|---|
| Team Selection (Play Now) | `A` (PLAY), `A` (PLAY NOW) |
| Quick Modes | `A` (PLAY), `↓` `↓` (QUICK MODES), `A` |
| Training | Quick Modes → `↓` `↓` `↓` (Training), `A` |
| Online | `A` (PLAY), `↓` `↓` `↓` (ONLINE), `A` |
| Career | `A` (PLAY), `↓` `↓` `↓` `↓` (CAREER), `A` |
| Tournaments | `A` (PLAY), `↓` `↓` `↓` `↓` `↓` (TOURNAMENTS), `A` |
| Customize | `↓` `↓` (CUSTOMIZE), `A` |
| Creation Zone | Customize → `A` (CREATION ZONE) |
| Roster Management | Customize → `↓` ×6 (ROSTER MANAGEMENT), `A` |
| Player Movement | Roster Management → `↓` (PLAYER MOVEMENT), `A` |
| Save/Load/Delete | Customize → `↓` ×7 (SAVE/LOAD/DELETE), `A` |
| Load Roster | Save/Load → `↓` (LOAD), `A` → `↓` (ROSTERS), `A` → select file, `A` → confirm Proceed |
| Settings | Customize → `↓` ×8 (SETTINGS), `A` |

### On-screen button hints

- **A** — Select / Advance
- **B** — Back
- **Y** — Game Manual
- **X** — REDEEM COINS
- **LT / RT** — Page navigation
- **RS** — Scroll Text
- Profile indicator: P 62

### Default focus

On the main menu, the cursor defaults to **COMMUNITY**. Navigate to other items with d-pad up/down.

## Navigation Hazards

### COMMUNITY flyout

COMMUNITY is a **flyout panel**, not a full-screen menu. When highlighted and `A` is pressed, its submenu expands in-place and **intercepts d-pad navigation** — pressing `↓` from COMMUNITY enters the submenu instead of moving to CUSTOMIZE. To reach CUSTOMIZE, close the flyout first with `B`.

- **Open**: `A` on COMMUNITY
- **Close**: `B` (single tap)
- **Auto-close**: After ~15–30s of idle, the flyout retracts
- **To reach CUSTOMIZE**: `B` to close flyout, then `↓`

### Menu wrapping

All vertical menus wrap around — scrolling past the last item returns to the first, and vice versa.

### Confirmation on back

Pressing `B` from the Team Selection screen triggers a Yes/No confirmation dialog. To actually go back: `↑` (select Yes), `A`.

## Input Timing

| Action | Wait after input |
|---|---|
| Menu item scroll (d-pad) | 0.5s |
| Screen transition (A to enter) | 2.0–3.0s |
| Screen transition (B to go back) | 1.5–2.0s |
| Game startup / title screens | up to 10s |

### Tap duration

Default `tap()` holds for 200ms (sufficient at 60fps). At low FPS (e.g., 10fps cap), use `tap_ms("btn", 300)` or `hold("btn", 0.3)` if inputs are dropped.

## Player Movement (Roster Trades)

The Player Movement screen (`Customize → ROSTER MANAGEMENT → PLAYER MOVEMENT`) is a **two-column layout** for swapping players between teams.

### Layout

```
┌─────────────────────┬─────────────────────┐
│  Left Panel (team)  │  Right Panel (team) │
│  ← d-pad left       │  d-pad right →      │
│  LB/RB: cycle team  │  LB/RB: cycle team  │
│  LT/RT: cycle league│  LT/RT: cycle league│
│                     │                     │
│  [player list]      │  [player list]      │
│                     │                     │
│  Moving Block       │  Moving Block       │
│  (staged player)    │  (staged player)    │
└─────────────────────┴─────────────────────┘
         X = Execute Move
```

### How to trade

1. **Switch panels**: `dpad_left` / `dpad_right` to highlight the desired column. The active panel shows a green indicator on the selected player.
2. **Cycle teams**: `LB` / `RB` to switch between teams within the current league.
3. **Cycle leagues**: `LT` / `RT` to switch between NHL, SHL, Free Agents, etc.
4. **Select player**: `A` on a player stages them in that panel's **Moving Block**.
5. **Switch panels** and repeat to stage a second player from the other team.
6. **Execute**: `X` to complete the swap.
7. **Drop to Free Agency**: `Y` instead of `X` sends the staged player to the Free Agent pool (one-sided, not a trade).

### Operational notes

- **d-pad left/right is NOT listed in on-screen hints** but is the only way to switch between the two panels. Without it you're stuck on one side.
- The roster file list under `LOAD → ROSTERS` shows saved rosters sorted by last-modified date (newest first). The first entry is highlighted by default.
- Loading a roster triggers a confirmation dialog ("All unsaved changes will be lost"). Navigate to **Proceed** (`↑` then `A`) to confirm.
- After loading, the game returns to the Roster file list; press `B` to back out to the Customize menu.

## Avoided Modes

| Mode | Reason |
|---|---|
| HOCKEY ULTIMATE TEAM | Complex setup, online-dependent |
| LIVE THE LIFE | Complex setup (Be A Pro variant) |
| NHL™ 94 ANNIVERSARY MODE | Complex setup |
| On-ice gameplay | Resource-intensive, derails menu mapping |

## Unexplored

- **COMMUNITY** submenu items (VIEW PLAYER HUB, LEADERBOARDS, LOBBY, MY HIGHLIGHTS)
- **CUSTOMIZE** leaf nodes: CUSTOMIZE AI, FAVORITE TEAM, OFFER CODE ENTRY, PROFILE MANAGEMENT, EA SPORTS™ MEDIA HUB
- **SETTINGS** submenu items
- **NHL™ Moments Live** internal menu (Game Modes, Season, Playoffs, Rosters, Settings, Credits, Quit)
- **Winter Classic** — venue selection only; gameplay entry not explored
- **ROSTER MANAGEMENT** leaf nodes: TEAM ROSTERS, EDIT LINES, JERSEY NUMBERS, SET DEFAULT ROSTERS, DOWNLOAD ROSTERS
- **SAVE/LOAD/DELETE** leaf nodes: SAVE, DELETE (only LOAD explored)
