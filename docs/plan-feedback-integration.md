# Plan: Game-State Feedback Integration for Menu Automation

## 1. Problem

Rhai scripts currently send Xbox controller inputs open-loop: `tap()`, `hold()`,
and `wait()` with no feedback from the game. Any timing deviation (loading
screens, frame drops, pop-ups) causes the script to fall out of sync with the
game state.

The primary use case is **deep menu-tree traversal for off-ice operations**:
roster editing, trade requests to CPU-controlled teams, and checking responses.
These require:

- Knowing **which screen** the game is on (depth in the menu tree)
- Knowing **what's selected** and **what's visible** (list items, focused index)
- **Making decisions** based on game state (e.g. "is this the right player?",
  "was the trade accepted?")
- **Navigating precisely** (scroll to items, select, go back, branch conditionally)

## 2. Analysis Summary

### 2.1. Game Logs — Not Viable

Game logs (`../NHL Legacy Recomp/logs/`, ~25K lines across 11 sessions) contain
only infrastructure-level output:

- Vulkan initialization / GPU enumeration
- Per-frame FPS counters (`[nhl-vk-fps]`)
- VFS file-not-found errors
- Database opens (`nhl_eng_us.db`, `nhlng.db`)
- Pipeline creation, Xam stubs, controller detection

There is **zero game-state information**: no menu names, scene transitions, UI
selection state, or flow-graph events.

### 2.2. Recomp Source — Feasible but Requires RE Work

The recomp source (`../nhl-legacy-recomp-experimental/`) has proven mechanisms
for intercepting guest functions and scanning guest memory:

| Mechanism | Example File |
|---|---|
| `REX_HOOK` function interception | `src/input_block.cpp` (already hooks `XamInputGetState`) |
| Guest memory scanning | `src/stick_list_scan.h`, `src/tunable_runtime.h` |
| File-based dumps | `src/tunable_registry_dump.h` |
| Per-frame overlay tick | `renderer/core/nhl_overlay.cpp:136` |

The challenge is that the game's UI/menu state is **not a single readable
variable**. It is driven by:

- A **ResourceKernel** (`resourcekernel.cpp`) managing a screen stack
- **Lua scripts** (`cache:\scrape\scenedef.lua`) defining scene composition
- **103,714 anonymous recompiled functions** with no symbol table

To expose "current screen = trade_response, selected item = 3", reverse
engineering is required to locate the ResourceKernel's screen-stack global and
screen-transition functions. This is a time investment, not a technical blocker.

**No existing IPC** mechanism exists in the recomp (no sockets, shared memory,
or named pipes). The recommended approach is to start with atomic file writes
(following the existing dump patterns), with the option to upgrade to a Unix
domain socket later.

## 3. Layered Approach

We adopt a **three-layer strategy**, where each layer adds robustness
incrementally and scripts can gracefully degrade when higher layers are
unavailable.

### Layer 1: Observer Trait Expansion + Script Wiring

| Component | Description |
|---|---|
| `Observer` trait expansion | Add methods: `current_screen()`, `menu_path()`, `selected_index()`, `visible_items()`, `context_value(key)` |
| `FileStateObserver` | Reads a JSON state file from a configurable path (e.g. `/tmp/nhl_state.json`) with atomic-read semantics |
| `NullObserver` | Remains the default — all queries return empty/neutral values so scripts work without feedback |
| Rhai host functions | Expose observer queries to scripts: `scene()`, `menu_path()`, `selected_index()`, `visible_items()`, `context(key)` |
| Navigation helpers | Rhai helper functions: `wait_for_scene(name, timeout)`, `select_item(name)`, `navigate_back()` |

Script engine signature changes from:

```rust
fn run_script(source: &str, controller: Arc<Mutex<Box<dyn Controller>>>)
```

to:

```rust
fn run_script(
    source: &str,
    controller: Arc<Mutex<Box<dyn Controller>>>,
    observer: Arc<dyn Observer>,
)
```

### Layer 2: Script-Level State Machine Primitives

Built-in Rhai helpers for declarative menu navigation, each using observer
data when available and falling back to timing when not:

```text
wait_for_scene(name, timeout)   — poll observer.current_screen() until match or timeout
navigate_to(path)                — follow a menu path like ["main_menu", "rosters", "edit"]
scroll_to_item(name)             — navigate a list until the target item is highlighted
select_item(name)                — scroll to + confirm selection of a named item
confirm_on_screen(name, timeout) — wait and verify we're on the expected screen
retry(action, check, max)        — generic retry with verification predicate
```

### Layer 3: Screen Capture Fallback (Optional, Deferred)

A `ScreenCaptureObserver` using lightweight pixel sampling (`pixel_at()`) as a
**fallback** when the recomp IPC is unavailable:

- Sample 5-20 strategic pixels per check (not full screenshots)
- Compare against known color signatures for menu highlights
- Goal: coarse verification ("are we on the expected screen?") — not content reading

This is explicitly a **fallback**, not the primary feedback mechanism. It is
fragile (resolution changes, UI theme changes) and cannot read dynamic content
like player names or trade responses. It is only useful for the specific case
where the recomp IPC is unavailable and a script needs coarse sanity checks.

## 4. Phased Implementation Plan

### Phase 1: Observer Trait + FileStateObserver + Script Wiring

**Goal:** Scripts can query the recomp's exported state via Rhai host functions.
The system works with `NullObserver` (no feedback) and upgrades seamlessly to
`FileStateObserver` (file-based IPC).

Steps:

1. Expand the `Observer` trait with new methods (`current_screen()`,
   `menu_path()`, `selected_index()`, `visible_items()`, `context_value(key)`).
   Provide default implementations returning empty/neutral values.

2. Implement `FileStateObserver` — reads a JSON state file from a configurable
   path, with atomic-read semantics (polling or inotify). Parses the schema the
   recomp will write.

3. Add `Arc<dyn Observer>` to the script engine (`run_script` signature change).

4. Register Rhai host functions: `scene()`, `menu_path()`, `selected_index()`,
   `visible_items()`, `context(key)`.

5. Update `main.rs` to wire the observer (choose `NullObserver` or
   `FileStateObserver` based on CLI flags).

6. Add `--observer null|file` CLI flag and `--observer-path` for the state file
   location.

7. Write example scripts demonstrating feedback-aware navigation.

### Phase 2: Script Navigation Helpers

**Goal:** A Rhai stdlib module with reusable menu-navigation primitives that
scripts can import.

Steps:

1. Implement `verify_scene(name, timeout)`, `wait_for_change(timeout)`,
   `wait_until_stable(consecutive_frames)`.

2. Implement `navigate_to(path)`, `select_item(name)`, `navigate_back()`.

3. Implement `retry(action_fn, check_fn, max_attempts)`.

4. Either bundle these as Rhai modules or pre-compile and inject them into the
   engine.

5. Write example scripts for roster editing and trade request flows.

### Phase 3: Recomp State Export (Recomp Side)

**Goal:** The recomp writes a JSON state file on every UI state change.

This phase lives in the recomp repo, not the automation tool. The automation
tool's `FileStateObserver` is designed to consume whatever the recomp produces,
and the schema can grow incrementally.

Initial state file schema:

```json
{
  "version": 1,
  "frame": 12345,
  "screen": "main_menu",
  "menu_path": ["main_menu"],
  "selected_index": 0,
  "visible_items": ["Play", "Rosters", "Settings"],
  "context": {}
}
```

Later schema additions:

```json
{
  "screen": "trade_response",
  "context": {
    "response_type": "counter_offer",
    "team_name": "Boston Bruins",
    "details": "..."
  }
}
```

The recomp-side implementation uses existing patterns:
- `REX_HOOK` for intercepting guest functions
- Guest memory scanning for state extraction
- Atomic file writes (write to temp, rename)
- Separate thread for periodic export

### Phase 4: Observer Upgrade Path (Optional)

**Goal:** Add `ScreenCaptureObserver` as a fallback for environments without
recomp IPC.

Only build this if a clear need emerges during real-world script development.

## 5. State File Protocol

### Location

Default: `~/.nhllegacy/state.json` (configurable via `--observer-path`).

The recomp writes to this path; the automation tool reads from it.

### Atomicity

The recomp writes state to a temp file (`state.json.tmp`) and atomically
renames it over the target. The automation tool's `FileStateObserver` detects
changes via polling or inotify and re-reads the file. Stale reads are
acceptable — the tool always reads the latest written state.

### Schema (v1)

```json
{
  "version": 1,
  "timestamp_ms": 1710441600123,
  "frame": 12345,
  "connected": true,
  "screen": "main_menu",
  "menu_path": ["main_menu"],
  "selected_index": 0,
  "visible_items": ["Play", "Rosters", "Settings"],
  "context": {}
}
```

| Field | Type | Description |
|---|---|---|
| `version` | int | Schema version for forward compatibility |
| `timestamp_ms` | int | Wall-clock timestamp (ms since epoch) |
| `frame` | int | Guest frame counter |
| `connected` | bool | Whether the game is running and responsive |
| `screen` | string | Current screen/menu identifier (e.g. `"main_menu"`, `"trade_response"`) |
| `menu_path` | []string | Path through the menu tree to the current screen |
| `selected_index` | int | Index of the currently focused menu item |
| `visible_items` | []string | Labels of visible menu items in the current list |
| `context` | object | Screen-specific freeform data (e.g. trade response details) |

### Extending the Schema

The schema is designed to grow. The automation tool's `FileStateObserver`
ignores unknown fields. New fields are added to support specific script needs
as they are identified during development.

## 6. Script Examples

### Feedback-aware menu navigation

```rhai
// Navigate from main menu to trade request screen
// Uses observer data when available; falls back to timing

fn main_menu_to_trade_request() {
    if scene() == "attract" {
        tap("a");
        verify_scene("main_menu", 5.0);
    }

    navigate_to(["main_menu", "rosters", "edit", "trade"]);

    // Scroll to find the target team
    let found = scroll_to_item("Boston Bruins");
    if !found {
        print("ERROR: team not found in list");
        navigate_back();
        return false;
    }

    tap("a");
    return verify_scene("trade_request", 3.0);
}
```

### Polling for game state changes

```rhai
// Wait for trade response, then branch on the result
loop {
    if scene() == "trade_response" {
        let response = context("response_type");
        if response == "accepted" {
            tap("a");  // confirm
            break;
        } else if response == "rejected" {
            tap("b");  // go back
            break;
        }
        // counter offer — handle elsewhere
    }
    wait(0.5);  // poll rate
}
```

## 7. Risk and Mitigation

| Risk | Mitigation |
|---|---|
| Recomp RE takes longer than expected | Scripts gracefully degrade to timing-based when observer returns empty values |
| State file schema evolves | Versioned schema; `FileStateObserver` ignores unknown fields |
| `NullObserver` is the default, so scripts may accidentally rely on feedback that isn't available | Navigation helpers print warnings when observer is a `NullObserver` and data is unavailable |
| File-based IPC adds latency | Polling at ~100ms intervals is sufficient for menu navigation (no real-time gameplay) |

## 8. Decision Log

| Decision | Rationale |
|---|---|
| Start with file-based IPC, not sockets | Simpler, follows existing recomp dump patterns, upgradable later |
| `FileStateObserver` + `NullObserver` as the two initial backends | One for production, one for development/testing |
| Pixel sampling deferred to Phase 4 | Fragile, cannot read dynamic content, recomp IPC is the primary path |
| Schema versioned from v1 | Forward compatibility as the recomp exposes more state |
| Navigation helpers in Rhai (not Rust) | Keeps them modifiable without recompilation; leverages Rhai's dynamic nature |
