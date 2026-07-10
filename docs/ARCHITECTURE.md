# Architecture

## Crates

```
crates/
├── controller/     Virtual gamepad backend trait + uinput impl
├── observer/       Game-state observation trait (feedback carveout)
├── script/         Rhai script engine + host function registration
└── app/            CLI binary (clap)
```

## Traits

### Controller (`crates/controller/src/traits.rs`)

```rust
trait Controller: Send {
    fn press(&mut self, button: Button) -> Result<(), ControllerError>;
    fn release(&mut self, button: Button) -> Result<(), ControllerError>;
    fn set_axis(&mut self, axis: Axis, value: f64) -> Result<(), ControllerError>;
    fn set_stick(&mut self, stick: Stick, x: f64, y: f64) -> Result<(), ControllerError>;
    fn flush(&mut self) -> Result<(), ControllerError>;
}
```

- **Linux**: `UinputController` creates an evdev virtual device with Xbox VID/PID.
- **Windows** (future): implement the trait against HIDMaestro or ViGEmBus.

### Observer (`crates/observer/src/lib.rs`)

```rust
trait Observer: Send + Sync {
    fn is_connected(&self) -> bool;
    fn detect_scene(&self) -> Scene;
    fn pixel_at(&self, x: u32, y: u32) -> Option<PixelColor>;
    fn template_match(&self, name: &str) -> Option<(f64, f64)>;
    fn capture_screenshot(&self, label: &str) -> anyhow::Result<String>;
    fn capture_screenshot_flat(&self, label: &str) -> anyhow::Result<String>;
}
```

- **ScreenCaptureObserver**: uses `xcap` to find the game window by substring match and capture screenshots. Supports `_run/` timestamped directories for script runs and flat saves for one-shot `--screenshot` mode.
- **NullObserver** ships as the default — all observations return empty/neutral.

### Script engine (`crates/script/src/lib.rs`)

Takes a Rhai script source string + `Arc<Mutex<Box<dyn Controller>>>`,
compiles and runs it. Registers `press`, `release`, `tap`, `hold`, `wait`,
`set_axis`, `set_stick`, `screenshot`, and `should_stop` as Rhai host functions.

The Observer trait is partially wired: `screenshot()` and `should_stop()` are
exposed to scripts. Candidates for future wiring: `pixel_at()`, `is_connected()`.

SIGINT (Ctrl+C) sets an `AtomicBool` flag. Rhai's `on_progress` callback checks
this flag on each operation and terminates the script cleanly. The `should_stop()`
Rhai function lets scripts check the flag voluntarily for graceful exit loops.

## Data flow

```
CLI ──▶ parse args ──▶ create UinputController
                  ──▶ read .rhai file
                  ──▶ run_script(source, controller, observer)
                         └── Rhai engine compiles + runs
                               └── host fns call Controller trait methods
                                     └── uinput emits evdev events
```
