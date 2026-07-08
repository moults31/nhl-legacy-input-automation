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
}
```

- **NullObserver** ships as the default — all observations return empty/neutral.
- **Future**: screen-capture + template matching (OpenCV), game-memory reading.

### Script engine (`crates/script/src/lib.rs`)

Takes a Rhai script source string + `Arc<Mutex<Box<dyn Controller>>>`,
compiles and runs it. Registers `press`, `release`, `tap`, `hold`, `wait`,
`set_axis`, `set_stick` as Rhai host functions.

## Data flow

```
CLI ──▶ parse args ──▶ create UinputController
                  ──▶ read .rhai file
                  ──▶ run_script(source, controller)
                         └── Rhai engine compiles + runs
                               └── host fns call Controller trait methods
                                     └── uinput emits evdev events
```

## Feedback carveout

The `Observer` trait is defined but not wired into the script engine yet.
When implemented, scripts will be able to:

```rhai
if observer.menu_visible() {
    tap("a");
}
```

The carveout requires no script-format or engine-architecture changes.
