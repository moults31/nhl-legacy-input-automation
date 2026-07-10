# NHL Legacy Menu Tree Map

*Generated via vision-guided exploration; screenshots in `screenshots/` run directories.*

## Root: Main Menu

```
MAIN MENU
├── PLAY                     [►]    (submenu; not yet explored)
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

These menus have been identified but not yet entered:

- **PLAY submenu** — PLAY is highlighted with [►] indicator. Pressing A while PLAY is in focus did not open its submenu during exploration. Likely needs different interaction.
- **CUSTOMIZE** — Not yet reached. COMMUNITY intercepts d-pad navigation when its submenu is expanded.

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

2. **Exiting the COMMUNITY flyout requires 5xB taps.** A single B tap does not close the flyout. Rapidly tapping B five times dismisses all layers and returns focus to the main menu item list.

3. **COMMUNITY flyout auto-closes after idle timeout.** After roughly 15–30 seconds without input, the COMMUNITY submenu retracts and the main menu items (PLAY, COMMUNITY, CUSTOMIZE) become fully navigable again.

4. **Menu list wraps.** Scrolling past the bottom of the COMMUNITY submenu wraps to the top. The main menu items (PLAY, COMMUNITY, CUSTOMIZE) also appear to wrap based on behavior observed.

5. **COMMUNITY is the default/focus item.** No matter which direction was tried (d-pad up, down, left stick), the cursor consistently settled on COMMUNITY when no submenu was active.

6. **PLAY submenu did not open with A press.** PLAY has the same [►] indicator as COMMUNITY, but pressing A while PLAY was focused did not open a submenu. Tried dpad_right as well — no effect. Needs further investigation.

7. **No system/pause menu accessible from main menu.** Pressing Start or Guide button from the main menu had no visible effect.

8. **LT/RT tab navigation did not work.** Attempted pressing left bumper to switch between PLAY/COMMUNITY/CUSTOMIZE as tabs — no response.

### Navigation Patterns Discovered

| Action | Effect |
|--------|--------|
| A on main menu item with [►] | Opens submenu |
| B (single) | Often ineffective for back navigation |
| B (rapid 5x) | Reliably returns from submenu to main menu |
| D-pad up/down | Vertical item selection (wraps) |
| Left stick up/down | Also works, but can overshoot |
| Idle >15s on COMMUNITY | Flyout auto-closes |
| D-pad right on [►] item | No observed effect |
