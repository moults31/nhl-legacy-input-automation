use std::fs;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Parser;
use nhl_controller::UinputController;
use nhl_observer::ScreenCaptureObserver;
use nhl_script::run_script;
use tracing::info;

#[derive(Parser)]
#[command(name = "nhl-input", about = "Virtual Xbox controller input automation")]
struct Cli {
    #[arg(short, long, default_value = "scripts/examples/spam-a-start.rhai")]
    script: String,

    #[arg(
        long,
        default_value = "NHL",
        help = "Substring to match in game window title for screenshots"
    )]
    window_substring: String,

    #[arg(long, help = "Take a single screenshot with this label and exit")]
    screenshot: Option<String>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let observer = ScreenCaptureObserver::new(&cli.window_substring);
    let observer: Arc<dyn nhl_observer::Observer> = Arc::new(observer);

    if let Some(label) = &cli.screenshot {
        let path = observer.capture_screenshot(label)?;
        eprintln!("screenshot saved: {path}");
        return Ok(());
    }

    let controller = UinputController::new().context("failed to create uinput controller")?;
    let controller: Arc<Mutex<Box<dyn nhl_controller::Controller>>> =
        Arc::new(Mutex::new(Box::new(controller)));

    info!("virtual Xbox controller created");

    let source = fs::read_to_string(&cli.script)
        .with_context(|| format!("failed to read script: {}", cli.script))?;
    info!(script = %cli.script, "loaded script");

    run_script(&source, controller, observer).context("script execution failed")?;

    info!("script finished");
    Ok(())
}
