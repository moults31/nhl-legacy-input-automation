# NHL Legacy Menu Tree Map

*Generated via vision-guided exploration; screenshots in `screenshots/` run directories.*

## Root: Main Menu

```
MAIN MENU
├── PLAY                     [►]    (submenu; explored below)
│   ├── PLAY NOW
│   ├── HOCKEY ULTIMATE TEAM
│   ├── QUICK MODES
│   ├── ONLINE
│   ├── CAREER
│   └── TOURNAMENTS
├── COMMUNITY                [►]    (submenu; explored below)
│   ├── VIEW PLAYER HUB
│   ├── LEADERBOARDS
│   ├── LOBBY
│   └── MY HIGHLIGHTS
├── CUSTOMIZE                        (not yet explored)
│
├── [FEATURED] ──────────────────────
│   ├── HOCKEY ULTIMATE TEAM         (complex mode — entrypoint noted, not explored)
│   ├── LIVE THE LIFE                (complex mode — entrypoint noted, not explored)
│   └── NHL™ 94 ANNIVERSARY MODE     (complex mode — entrypoint noted, not explored)
```

### On-screen button hints

- **Y** — Game Manual
- **LT / RT** — 1/2 (tab/page navigation)
- **RS** — Scroll Text
- **X** — REDEEM COINS
- Profile indicator: P 62

---

## Exploration TBD

These menus have been identified but not yet fully explored:

- **PLAY submenu** — PLAY NOW, HOCKEY ULTIMATE TEAM, QUICK MODES, ONLINE, CAREER, TOURNAMENTS. Entry confirmed; sub-items not yet explored.
- **CUSTOMIZE** — Not yet reached.

---

## Avoided (by design)

| Mode | Reason |
|------|--------|
| HOCKEY ULTIMATE TEAM | Complex setup, online-dependent |
| LIVE THE LIFE | Complex setup (Be A Pro variant) |
| NHL™ 94 ANNIVERSARY MODE | Complex setup |
| On-ice gameplay | Resource-intensive, derails menu mapping |

---

## Operational Notes

### Friction & Surprises

1. **COMMUNITY submenu is a persistent flyout panel.** When COMMUNITY is highlighted and A is pressed, its submenu (VIEW PLAYER HUB, LEADERBOARDS, LOBBY, MY HIGHLIGHTS) expands in place — not a full screen transition. This submenu intercepts d-pad navigation: scrolling down from COMMUNITY enters the submenu rather than moving to CUSTOMIZE.

2. **Exiting the COMMUNITY flyout requires a single B tap.** A single B press closes the flyout and returns focus to the main menu item list. (Previously reported as requiring 5xB taps — this was caused by the 16ms tap duration being too short for the game to register.)

3. **COMMUNITY flyout auto-closes after idle timeout.** After roughly 15–30 seconds without input, the COMMUNITY submenu retracts and the main menu items (PLAY, COMMUNITY, CUSTOMIZE) become fully navigable again.

4. **Menu list wraps.** Scrolling past the bottom of the COMMUNITY submenu wraps to the top. The main menu items (PLAY, COMMUNITY, CUSTOMIZE) also appear to wrap based on behavior observed.

5. **COMMUNITY is the default/focus item.** No matter which direction was tried (d-pad up, down, left stick), the cursor consistently settled on COMMUNITY when no submenu was active.

6. **PLAY submenu opens with a single A press.** PLAY has the same [►] indicator as COMMUNITY, and pressing A while PLAY is focused opens its submenu (PLAY NOW, HOCKEY ULTIMATE TEAM, QUICK MODES, ONLINE, CAREER, TOURNAMENTS). (Previously reported as not opening — this was caused by the 16ms tap duration being too short.)

7. **No system/pause menu accessible from main menu.** Pressing Start from the main menu has no visible effect. This is by design — the pause menu is only available during gameplay.

8. **LT/RT cycles page indicators but does not navigate main items.** The page counter (e.g., "1/2", "2/3") changes, but the main menu item list (PLAY, COMMUNITY, CUSTOMIZE) remains the same. Use d-pad up/down to navigate between items.

9. **Tap duration was the root cause of dropped inputs.** The original 16ms tap duration was too short for reliable input registration. Increasing to 200ms (spanning ~12 frames at 60fps, ~2 frames at 10fps) resolved the PLAY submenu and COMMUNITY flyout issues. At 10fps, if inputs are still dropped, use `tap_ms("btn", 300)` or `hold("btn", 0.3)`.

### Navigation Patterns Discovered

| Action | Effect |
|--------|--------|
| A on main menu item with [►] | Opens submenu |
| B (single) | Closes flyout/submenu, returns to main menu |
| D-pad up/down | Vertical item selection (wraps) |
| Left stick up/down | Also works, but can overshoot |
| Idle >15s on COMMUNITY | Flyout auto-closes |
| D-pad right on [►] item | No observed effect |
| LT/RT | Cycles page indicator; does not change main items |
