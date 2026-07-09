use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use nhl_controller::{Button, Controller};
use nhl_observer::Observer;
use rhai::{Dynamic, Engine, Scope};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn request_stop() {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

pub fn run_script(
    source: &str,
    controller: Arc<Mutex<Box<dyn Controller>>>,
    observer: Arc<dyn Observer>,
) -> Result<()> {
    let mut engine = Engine::new();

    engine.set_max_operations(1_000_000_000);

    engine.on_progress(|_count| {
        if INTERRUPTED.load(Ordering::Relaxed) {
            Some(Dynamic::from("interrupted"))
        } else {
            None
        }
    });

    {
        let ctrl = Arc::clone(&controller);
        engine.register_fn("press", move |button: &str| {
            let btn = parse_button(button)?;
            let mut c = ctrl.lock().map_err(|e| format!("mutex lock: {e}"))?;
            c.press(btn).map_err(|e| format!("press: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))
        });
    }

    {
        let ctrl = Arc::clone(&controller);
        engine.register_fn("release", move |button: &str| {
            let btn = parse_button(button)?;
            let mut c = ctrl.lock().map_err(|e| format!("mutex lock: {e}"))?;
            c.release(btn).map_err(|e| format!("release: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))
        });
    }

    {
        let ctrl = Arc::clone(&controller);
        engine.register_fn("tap", move |button: &str| {
            let btn = parse_button(button)?;
            let mut c = ctrl.lock().map_err(|e| format!("mutex lock: {e}"))?;
            c.press(btn).map_err(|e| format!("press: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))?;
            std::thread::sleep(Duration::from_millis(16));
            c.release(btn).map_err(|e| format!("release: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))
        });
    }

    {
        let ctrl = Arc::clone(&controller);
        engine.register_fn("hold", move |button: &str, duration: f64| {
            let btn = parse_button(button)?;
            let mut c = ctrl.lock().map_err(|e| format!("mutex lock: {e}"))?;
            c.press(btn).map_err(|e| format!("press: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))?;
            let dur = Duration::from_secs_f64(duration);
            std::thread::sleep(dur);
            c.release(btn).map_err(|e| format!("release: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))
        });
    }

    {
        let ctrl = Arc::clone(&controller);
        engine.register_fn("set_axis", move |axis: &str, value: f64| {
            let ax = parse_axis(axis)?;
            let mut c = ctrl.lock().map_err(|e| format!("mutex lock: {e}"))?;
            c.set_axis(ax, value)
                .map_err(|e| format!("set_axis: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))
        });
    }

    {
        let ctrl = Arc::clone(&controller);
        engine.register_fn("set_stick", move |stick: &str, x: f64, y: f64| {
            let st = parse_stick(stick)?;
            let mut c = ctrl.lock().map_err(|e| format!("mutex lock: {e}"))?;
            c.set_stick(st, x, y)
                .map_err(|e| format!("set_stick: {e}"))?;
            c.flush().map_err(|e| format!("flush: {e}"))
        });
    }

    engine.register_fn("wait", |duration: f64| {
        std::thread::sleep(Duration::from_secs_f64(duration));
    });

    {
        let obs = Arc::clone(&observer);
        engine.register_fn("screenshot", move |label: &str| -> Result<String, String> {
            obs.capture_screenshot(label)
                .map_err(|e| format!("screenshot: {e}"))
        });
    }

    engine.register_fn("should_stop", || -> bool {
        INTERRUPTED.load(Ordering::Relaxed)
    });

    let ast = engine.compile(source).map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut scope = Scope::new();
    let result = engine.run_ast_with_scope(&mut scope, &ast);

    match result {
        Ok(()) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("interrupted") {
                tracing::info!("script interrupted by user");
                Ok(())
            } else {
                Err(anyhow::anyhow!("{err}"))
            }
        }
    }
}

fn parse_button(name: &str) -> Result<Button, String> {
    match name.to_lowercase().as_str() {
        "a" => Ok(Button::A),
        "b" => Ok(Button::B),
        "x" => Ok(Button::X),
        "y" => Ok(Button::Y),
        "start" => Ok(Button::Start),
        "back" | "select" => Ok(Button::Back),
        "left_bumper" | "lb" => Ok(Button::LeftBumper),
        "right_bumper" | "rb" => Ok(Button::RightBumper),
        "left_thumb" | "l3" => Ok(Button::LeftThumb),
        "right_thumb" | "r3" => Ok(Button::RightThumb),
        "guide" | "xbox" => Ok(Button::Guide),
        "dpad_up" | "up" => Ok(Button::DpadNorth),
        "dpad_down" | "down" => Ok(Button::DpadSouth),
        "dpad_left" | "left" => Ok(Button::DpadWest),
        "dpad_right" | "right" => Ok(Button::DpadEast),
        _ => Err(format!("unknown button: {name}")),
    }
}

fn parse_axis(name: &str) -> Result<nhl_controller::Axis, String> {
    match name.to_lowercase().as_str() {
        "left_stick_x" | "lsx" => Ok(nhl_controller::Axis::LeftStickX),
        "left_stick_y" | "lsy" => Ok(nhl_controller::Axis::LeftStickY),
        "right_stick_x" | "rsx" => Ok(nhl_controller::Axis::RightStickX),
        "right_stick_y" | "rsy" => Ok(nhl_controller::Axis::RightStickY),
        "left_trigger" | "lt" => Ok(nhl_controller::Axis::LeftTrigger),
        "right_trigger" | "rt" => Ok(nhl_controller::Axis::RightTrigger),
        "dpad_x" | "dx" => Ok(nhl_controller::Axis::DpadX),
        "dpad_y" | "dy" => Ok(nhl_controller::Axis::DpadY),
        _ => Err(format!("unknown axis: {name}")),
    }
}

fn parse_stick(name: &str) -> Result<nhl_controller::Stick, String> {
    match name.to_lowercase().as_str() {
        "left" => Ok(nhl_controller::Stick::Left),
        "right" => Ok(nhl_controller::Stick::Right),
        _ => Err(format!("unknown stick: {name}")),
    }
}
