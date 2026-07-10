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
│   ├── ROSTER MANAGEMENT
│   ├── SAVE/LOAD/DELETE
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

### Virtual controller warm-up

The virtual Xbox controller device is created on each `nhl-input` invocation and destroyed on exit. The game needs ~3s to recognize the new device. Always start `-e` scripts with `wait(3.0)`.

```
# Correct:
nhl-input -e 'wait(3.0); tap("a"); wait(2.0); ...'

# Wrong — inputs may be silently dropped:
nhl-input -e 'tap("a"); ...'
```

### Tap duration

Default `tap()` holds for 200ms (sufficient at 60fps). At low FPS (e.g., 10fps cap), use `tap_ms("btn", 300)` or `hold("btn", 0.3)` if inputs are dropped.

## Avoided Modes

| Mode | Reason |
|---|---|
| HOCKEY ULTIMATE TEAM | Complex setup, online-dependent |
| LIVE THE LIFE | Complex setup (Be A Pro variant) |
| NHL™ 94 ANNIVERSARY MODE | Complex setup |
| On-ice gameplay | Resource-intensive, derails menu mapping |

## Unexplored

- **COMMUNITY** submenu items (VIEW PLAYER HUB, LEADERBOARDS, LOBBY, MY HIGHLIGHTS)
- **CUSTOMIZE** leaf nodes: CUSTOMIZE AI, FAVORITE TEAM, OFFER CODE ENTRY, PROFILE MANAGEMENT, ROSTER MANAGEMENT, SAVE/LOAD/DELETE, EA SPORTS™ MEDIA HUB
- **SETTINGS** submenu items
- **NHL™ Moments Live** internal menu (Game Modes, Season, Playoffs, Rosters, Settings, Credits, Quit)
- **Winter Classic** — venue selection only; gameplay entry not explored
