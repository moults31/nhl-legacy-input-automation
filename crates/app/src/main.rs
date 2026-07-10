use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Parser;
use nhl_controller::UinputController;
use nhl_observer::{Observer, ScreenCaptureObserver};
use nhl_script::run_script;
use tracing::info;

#[derive(Parser)]
#[command(name = "nhl-input", about = "Virtual Xbox controller input automation")]
struct Cli {
    #[arg(short, long, help = "Path to a Rhai script file")]
    script: Option<String>,

    #[arg(
        short = 'e',
        long,
        conflicts_with = "script",
        help = "Execute inline Rhai code"
    )]
    eval: Option<String>,

    #[arg(
        long,
        default_value = "NHL",
        help = "Substring to match in game window title for screenshots"
    )]
    window_substring: String,

    #[arg(long, help = "Take a single screenshot with this label and exit")]
    screenshot: Option<String>,

    #[arg(long, help = "List all visible window titles and exit")]
    list_windows: bool,

    #[arg(
        long,
        default_value = "screenshots/latest.png",
        help = "Continuously update this file with the latest screenshot"
    )]
    watch: Option<String>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let screen_observer = Arc::new(ScreenCaptureObserver::new(&cli.window_substring));

    if cli.list_windows {
        let windows = xcap::Window::all()?;
        for w in &windows {
            if let Ok(t) = w.title() {
                println!("{t}");
            }
        }
        return Ok(());
    }

    if let Some(label) = &cli.screenshot {
        let path = screen_observer.capture_screenshot_flat(label)?;
        eprintln!("screenshot saved: {path}");
        return Ok(());
    }

    if let Some(ref watch_path) = cli.watch {
        let p = PathBuf::from(watch_path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        screen_observer.set_watch(p);
    }

    let observer: Arc<dyn nhl_observer::Observer> = screen_observer;

    ctrlc::set_handler(|| {
        nhl_script::request_stop();
    })
    .context("failed to set Ctrl+C handler")?;

    let controller = UinputController::new().context("failed to create uinput controller")?;
    let controller: Arc<Mutex<Box<dyn nhl_controller::Controller>>> =
        Arc::new(Mutex::new(Box::new(controller)));

    info!("virtual Xbox controller created");

    let source = match (&cli.script, &cli.eval) {
        (None, None) => anyhow::bail!("either --script or --eval must be provided"),
        (Some(path), None) => {
            fs::read_to_string(path).with_context(|| format!("failed to read script: {path}"))?
        }
        (None, Some(inline)) => inline.clone(),
        _ => unreachable!(),
    };
    let source_label = cli.script.as_deref().unwrap_or("<inline>");
    info!(script = %source_label, "loaded script");

    run_script(&source, controller, observer).context("script execution failed")?;

    info!("script finished");
    Ok(())
}
