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
│   │   ├── BE A GM MODE
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

### Main menu variant (legacy/simplified)

An older-style main menu appears depending on game version or context:

```
LEGACY MAIN MENU
├── PLAY NOW
├── GAME MODES                     [A]  → Grid: PLAY NOW, SEASON, BE A GM, BE A PRO, TOURNAMENT, NHL 94, PRACTICE, MY NHL
│   └── Each mode → Title Screen → Main Menu (modern version after Start + Autosave)
├── NHL 94 ANNIVERSARY
├── MY NHL 15
├── CUSTOMIZE
├── SETTINGS
└── REPLAY
```

**Transition between variants:** Entering a game mode from the legacy menu
leads through a title screen, which after Start + Autosave dismissal opens
the modern Main Menu. Agents should not be confused by the two main menu
styles — they share the same submenu structure.

## Navigation Reference

### How to reach a destination

| Destination | Path from Main Menu (modern) |
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

### On-screen button hints

- **A** — Select / Advance
- **B** — Back
- **Y** — Game Manual
- **X** — REDEEM COINS
- **LT / RT** — Page navigation
- **RS** — Scroll Text

### Default focus

On the modern Main Menu, the cursor defaults to **COMMUNITY** on first
entry. Navigate to other items with d-pad up/down.

## Player Movement (Roster Trades)

The Player Movement screen (`Customize → ROSTER MANAGEMENT → PLAYER MOVEMENT`)
is a **two-column layout** for swapping players between teams.

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
- The roster file list under `LOAD → ROSTERS` shows saved rosters sorted by
  last-modified date (newest first). The first entry is highlighted by
  default.
- Loading a roster triggers a confirmation dialog ("All unsaved changes will
  be lost"). Navigate to **Proceed** (`↑` then `A`) to confirm.
- After loading, the game returns to the Roster file list; press `B` to
  back out to the Customize menu.

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

**Triggers at low FPS:** Under a 10fps cap, `tap("rt")` or `tap("lt")` can
be dropped. Use explicit axis setting with a hold duration instead:

```rhai
set_axis("right_trigger", 1.0);
wait(0.3);
set_axis("right_trigger", 0.0);
wait(0.8);
```

## Operational Gotchas

- **First launch Autosave dialog**: Appears after pressing Start on the
  title screen. Must be dismissed with A (Okay) before menus are reachable.
- **Title screen**: Requires Start to advance, not A. Other screens accept A.
- **Two main menu versions**: The legacy menu (PLAY NOW, GAME MODES, etc.)
  and the modern menu (PLAY, COMMUNITY, CUSTOMIZE, etc.). They share the
  same submenu tree. Navigating into a game mode from the legacy menu then
  exiting lands on the modern menu — not the legacy one.
- **Game process tree**: Three processes (python3 proton launcher →
  steam.exe → nhllegacy.exe). Kill all three to fully stop the game. Use
  `scripts/kill-nhl.sh`.
- **Game window substring**: `"nhllegacy"` matches the Proton window. Title
  contains "rexglue-v..." version string.
- **Vision-confusable screens**: The two main menu variants look different
  but lead to the same destinations. The vision model may report them
  separately. Consult this map for canonical navigation paths.

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
- Legacy Main Menu submenus (GAME MODES grid destinations beyond mode selection)


## Operational Gotchas (Added by EXECUTE agent)

- **Daemon input**: When chaining `tap()` and `wait()` commands in a single `--send` call, the daemon sometimes silently drops inputs. Send inputs one-by-one with screenshots between each step.
