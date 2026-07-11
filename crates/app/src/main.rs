use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use nhl_controller::UinputController;
use nhl_observer::{Observer, ScreenCaptureObserver};
use nhl_script::run_script;
use serde_json::Value;
use tracing::{error, info};

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

    #[arg(
        long,
        help = "Take a single screenshot with this label and exit. \
                When used with --log-step, this is the path to an existing screenshot file."
    )]
    screenshot: Option<String>,

    #[arg(long, help = "List all visible window titles and exit")]
    list_windows: bool,

    #[arg(
        long,
        default_value = "screenshots/latest.png",
        help = "Continuously update this file with the latest screenshot"
    )]
    watch: Option<String>,

    #[arg(
        long,
        help = "Screenshot directory name under screenshots/ (e.g. '20260709_120000_explore'). \
                When set, used verbatim as the directory name; when absent, auto-generated with a timestamp."
    )]
    run_id: Option<String>,

    #[arg(
        long,
        conflicts_with_all = ["script", "eval", "screenshot", "send"],
        help = "Start in daemon mode, keeping the virtual controller alive between commands"
    )]
    daemon: bool,

    #[arg(
        long,
        default_value = "_data/nhl-input.sock",
        help = "Unix socket path (bound by --daemon, used by --send)"
    )]
    socket: String,

    #[arg(
        long,
        conflicts_with_all = ["script", "eval", "screenshot", "list_windows", "daemon"],
        help = "Send inline Rhai code to a running daemon via the socket"
    )]
    send: Option<String>,

    #[arg(
        long,
        conflicts_with_all = ["script", "eval", "list_windows", "daemon", "send"],
        help = "Append a validated log entry to run_log.jsonl and exit"
    )]
    log_step: bool,

    #[arg(
        long,
        requires = "log_step",
        help = "Step number in the current execution sequence"
    )]
    step: Option<u32>,

    #[arg(
        long,
        requires = "log_step",
        help = "Path to a file containing the full vision prompt sent to the vision model"
    )]
    prompt_file: Option<String>,

    #[arg(
        long,
        requires = "log_step",
        help = "Path to a file containing the raw JSON response from the vision model"
    )]
    response_file: Option<String>,

    #[arg(
        long,
        requires = "log_step",
        help = "Agent assessment: match_confirmed, mismatch, recovery, or halt"
    )]
    assessment: Option<String>,

    #[arg(
        long,
        requires = "log_step",
        help = "Agent decision: navigate, recover, or halt"
    )]
    decision: Option<String>,

    #[arg(
        long,
        requires = "log_step",
        help = "One-line summary of what input will be sent next and why"
    )]
    plan: Option<String>,

    #[arg(
        long,
        help = "With --daemon: write command and screenshot events to daemon_events.jsonl"
    )]
    log_json: bool,

    #[arg(
        long,
        help = "With --daemon: disable step-logging enforcement. \
                Without this flag, the daemon rejects --send commands if \
                the previous step was not logged via --log-step. \
                This flag exists for manual debugging; never use it in \
                automated navigation."
    )]
    no_require_logging: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    if cli.list_windows {
        let windows = xcap::Window::all()?;
        for w in &windows {
            if let Ok(t) = w.title() {
                println!("{t}");
            }
        }
        return Ok(());
    }

    if let Some(ref send_script) = cli.send {
        return send_to_daemon(&cli.socket, send_script);
    }

    if cli.log_step {
        return run_log_step(&cli);
    }

    if cli.daemon {
        return run_daemon(&cli);
    }

    let screen_observer = Arc::new(ScreenCaptureObserver::new(&cli.window_substring));

    if let Some(ref run_id) = cli.run_id {
        screen_observer.set_run_id(run_id);
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

fn run_log_step(cli: &Cli) -> Result<()> {
    let run_id = cli
        .run_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--run-id is required"))?;
    let step = cli
        .step
        .ok_or_else(|| anyhow::anyhow!("--step is required"))?;
    let screenshot = cli
        .screenshot
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--screenshot (path) is required"))?;
    let prompt_file = cli
        .prompt_file
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--prompt-file is required"))?;
    let response_file = cli
        .response_file
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--response-file is required"))?;
    let assessment = cli
        .assessment
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--assessment is required"))?;
    let decision = cli
        .decision
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--decision is required"))?;
    let plan = cli
        .plan
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--plan is required"))?;

    let valid_assessments = [
        "goal_match",
        "goal_mismatch",
        "inconsistent",
        "recovery",
        "halt",
    ];
    if !valid_assessments.contains(&assessment.as_str()) {
        anyhow::bail!("--assessment must be one of: {:?}", valid_assessments);
    }

    let valid_decisions = ["navigate", "recover", "halt"];
    if !valid_decisions.contains(&decision.as_str()) {
        anyhow::bail!("--decision must be one of: {:?}", valid_decisions);
    }

    let screenshot_path = PathBuf::from(screenshot);
    if !screenshot_path.exists() {
        anyhow::bail!("screenshot file does not exist: {}", screenshot);
    }

    let prompt_text = fs::read_to_string(prompt_file)
        .with_context(|| format!("failed to read prompt file: {prompt_file}"))?;

    let banned_prompt_patterns: &[&str] = &[
        "Look at this screenshot file",
        "You are navigating the menu system",
        "I am executing a task in",
        "EXPECTS this screenshot to show",
        "MATCH question",
        "actual_screen_title",
        r#"{"match":"#,
        r#""match": true|false"#,
    ];
    for pattern in banned_prompt_patterns {
        if prompt_text.contains(pattern) {
            anyhow::bail!(
                "vision prompt contains banned pattern: {:?}\n\
                 The vision prompt must be the pure unified prompt from the skill — \
                 no task context, no expected screen, no match question. \
                 See SKILL.md §6 for the canonical template.",
                pattern
            );
        }
    }

    // Validate the Screenshot: header line in the prompt. It must contain only
    // a bare filename (NNN_step_N.png), not a directory path. Including the
    // run_id directory in the path leaks task context to the vision model and
    // causes hallucination.
    for line in prompt_text.lines() {
        if let Some(filename) = line.strip_prefix("Screenshot: ") {
            let filename = filename.trim();
            if filename.is_empty() {
                anyhow::bail!(
                    "vision prompt 'Screenshot:' header must specify a filename \
                     (e.g. 'Screenshot: 001_step_001.png'), got: {:?}",
                    line
                );
            }
            if filename.contains('/') {
                anyhow::bail!(
                    "vision prompt 'Screenshot:' header must contain ONLY a bare \
                     filename (e.g. 'Screenshot: 001_step_001.png'), not a full \
                     path. Including a directory path leaks run_id context to the \
                     vision model. Got: {:?}",
                    filename
                );
            }
            break;
        }
    }

    // Validate RUN_ID neutrality. Descriptive suffixes like _trade_mtl_tor
    // leak task context into directory names, which the vision model can
    // discover through the screenshot path in daemon_events.jsonl diagnostics.
    validate_run_id_neutral(run_id)?;

    let response_text = fs::read_to_string(response_file)
        .with_context(|| format!("failed to read response file: {response_file}"))?;

    let response_value: Value =
        serde_json::from_str(&response_text).with_context(|| "response file is not valid JSON")?;

    let required_fields: &[&str] = &[
        "all_text",
        "screen_title",
        "layout",
        "layout_description",
        "options",
        "selected",
        "button_hints",
        "gameplay",
        "confidence",
    ];
    for field in required_fields {
        if response_value.get(field).is_none() {
            anyhow::bail!("vision response missing required field: {}", field);
        }
    }

    // --- type validation ---

    if !response_value["all_text"].is_array() {
        anyhow::bail!("all_text must be an array");
    }
    for (i, v) in response_value["all_text"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        if !v.is_string() {
            anyhow::bail!("all_text[{}] must be a string", i);
        }
    }

    if !response_value["screen_title"].is_string() {
        anyhow::bail!("screen_title must be a string");
    }

    if !response_value["layout_description"].is_string() {
        anyhow::bail!("layout_description must be a string");
    }

    let valid_layouts = ["list", "two_column", "tabs", "grid", "custom"];
    let layout = response_value["layout"].as_str().unwrap_or("");
    if !valid_layouts.contains(&layout) {
        anyhow::bail!("layout must be one of {:?}, got: {}", valid_layouts, layout);
    }

    if !response_value["options"].is_array() {
        anyhow::bail!("options must be an array");
    }
    for (i, v) in response_value["options"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        if !v.is_string() {
            anyhow::bail!("options[{}] must be a string", i);
        }
    }

    if !response_value["selected"].is_string() {
        anyhow::bail!("selected must be a string");
    }

    if !response_value["button_hints"].is_array() {
        anyhow::bail!("button_hints must be an array");
    }
    for (i, v) in response_value["button_hints"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        if !v.is_string() {
            anyhow::bail!("button_hints[{}] must be a string", i);
        }
    }

    if !response_value["gameplay"].is_boolean() {
        anyhow::bail!("gameplay must be a boolean");
    }

    let valid_confidences = ["high", "medium", "low"];
    let confidence = response_value["confidence"].as_str().unwrap_or("");
    if !valid_confidences.contains(&confidence) {
        anyhow::bail!(
            "confidence must be one of {:?}, got: {}",
            valid_confidences,
            confidence
        );
    }

    // Optional breadcrumbs: if present, must be a string
    if let Some(b) = response_value.get("breadcrumbs") {
        if !b.is_string() {
            anyhow::bail!("breadcrumbs must be a string if present");
        }
    }

    // Optional regions: if present, must be an array of objects with string fields
    if let Some(regions) = response_value.get("regions") {
        if !regions.is_array() {
            anyhow::bail!("regions must be an array if present");
        }
        for (i, region) in regions.as_array().unwrap().iter().enumerate() {
            if !region.is_object() {
                anyhow::bail!("regions[{}] must be an object", i);
            }
            for field in &["name", "options", "selected"] {
                if region.get(field).is_none() {
                    anyhow::bail!("regions[{}] missing required field: {}", i, field);
                }
            }
            if !region["name"].is_string() {
                anyhow::bail!("regions[{}].name must be a string", i);
            }
            if !region["options"].is_array() {
                anyhow::bail!("regions[{}].options must be an array", i);
            }
            if !region["selected"].is_string() {
                anyhow::bail!("regions[{}].selected must be a string", i);
            }
            for (j, opt) in region["options"].as_array().unwrap().iter().enumerate() {
                if !opt.is_string() {
                    anyhow::bail!("regions[{}].options[{}] must be a string", i, j);
                }
            }
        }
    }

    let log_entry = serde_json::json!({
        "step": step,
        "screenshot": screenshot,
        "vision_prompt": prompt_text,
        "vision_response": response_value,
        "assessment": assessment,
        "decision": decision,
        "plan": plan,
    });

    let run_dir = PathBuf::from("screenshots").join(run_id);
    fs::create_dir_all(&run_dir)?;

    let log_path = run_dir.join("run_log.jsonl");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    serde_json::to_writer(&mut file, &log_entry)?;
    file.write_all(b"\n")?;

    eprintln!("log-step: step {} logged to {}", step, log_path.display());
    Ok(())
}

fn run_daemon(cli: &Cli) -> Result<()> {
    if let Ok(stream) = UnixStream::connect(&cli.socket) {
        drop(stream);
        anyhow::bail!(
            "A daemon is already running on {socket}. Kill it first:\n  kill $(pgrep -f \"nhl-input --daemon\")",
            socket = cli.socket
        );
    }
    let _ = fs::remove_file(&cli.socket);

    let socket_path = PathBuf::from(&cli.socket);
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let screen_observer = Arc::new(ScreenCaptureObserver::new(&cli.window_substring));
    let sc_observer_for_counter = Arc::clone(&screen_observer);

    if let Some(ref run_id) = cli.run_id {
        screen_observer.set_run_id(run_id);
    }

    if let Some(ref watch_path) = cli.watch {
        let p = PathBuf::from(watch_path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        screen_observer.set_watch(p);
    }

    let json_log: Option<Arc<Mutex<BufWriter<fs::File>>>> = if cli.log_json {
        let run_id = cli.run_id.as_deref().unwrap_or("unknown");
        let dir = PathBuf::from("screenshots").join(run_id);
        fs::create_dir_all(&dir)?;
        let log_path = dir.join("daemon_events.jsonl");
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        let writer = Arc::new(Mutex::new(BufWriter::new(file)));
        screen_observer.set_json_log(Arc::clone(&writer));
        info!("JSON event log: {}", log_path.display());
        Some(writer)
    } else {
        None
    };

    let observer: Arc<dyn nhl_observer::Observer> = screen_observer;

    let controller = UinputController::new().context("failed to create uinput controller")?;
    let controller: Arc<Mutex<Box<dyn nhl_controller::Controller>>> =
        Arc::new(Mutex::new(Box::new(controller)));

    info!("virtual Xbox controller created, warming up device (3s)...");
    std::thread::sleep(Duration::from_secs(3));
    info!("warmup complete, ready for commands");

    let listener = UnixListener::bind(&cli.socket)
        .with_context(|| format!("failed to bind daemon socket: {}", cli.socket))?;
    info!("daemon listening on {}", cli.socket);

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => {
                let mut buf = String::new();
                {
                    let mut reader = BufReader::new(&mut stream);
                    if let Err(e) = reader.read_line(&mut buf) {
                        error!("failed to read command from socket: {e}");
                        continue;
                    }
                }
                let script = buf.trim();
                if script.is_empty() {
                    continue;
                }

                info!(%script, "daemon executing command");

                if let Some(ref log) = json_log {
                    let ts =
                        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                    let event = serde_json::json!({
                        "ts": ts,
                        "event": "command",
                        "script": script,
                    });
                    if let Ok(mut writer) = log.lock() {
                        let _ = serde_json::to_writer(&mut *writer, &event);
                        let _ = writer.write_all(b"\n");
                        let _ = writer.flush();
                    }
                }

                if !cli.no_require_logging {
                    if let Some(ref run_id) = cli.run_id {
                        let log_path = PathBuf::from("screenshots")
                            .join(run_id)
                            .join("run_log.jsonl");
                        let log_count = if log_path.exists() {
                            fs::read_to_string(&log_path)
                                .unwrap_or_default()
                                .lines()
                                .filter(|l| !l.trim().is_empty())
                                .count()
                        } else {
                            0
                        };
                        let screenshot_count = sc_observer_for_counter.counter();
                        if screenshot_count.saturating_sub(log_count as u32) >= 2 {
                            let msg = format!(
                                "UNLOGGED_STEP: Must call --log-step before sending next command. \
                                 screenshots_taken={screenshot_count}, logged_steps={log_count}"
                            );
                            error!(%msg, "rejecting command due to logging enforcement");
                            let response = serde_json::json!({
                                "ok": false,
                                "err": msg,
                            });
                            if let Ok(canonical) = serde_json::to_string(&response) {
                                let _ = stream.write_all(canonical.as_bytes());
                                let _ = stream.write_all(b"\n");
                                let _ = stream.flush();
                            }
                            continue;
                        }
                    }
                }

                let result = run_script(script, Arc::clone(&controller), Arc::clone(&observer));

                let response = match result {
                    Ok(()) => r#"{"ok":true}"#.to_string(),
                    Err(e) => {
                        let escaped = json_escape(&format!("{e:#}"));
                        format!(r#"{{"ok":false,"err":"{}"}}"#, escaped)
                    }
                };

                if let Err(e) = stream
                    .write_all(response.as_bytes())
                    .and_then(|_| stream.write_all(b"\n"))
                    .and_then(|_| stream.flush())
                {
                    error!("failed to send daemon response: {e}");
                }
            }
            Err(e) => {
                error!("daemon accept error: {e}");
            }
        }
    }

    let _ = fs::remove_file(&cli.socket);
    info!("daemon shut down");
    Ok(())
}

fn validate_run_id_neutral(run_id: &str) -> Result<()> {
    let valid = run_id.len() == 19
        && run_id[..8].chars().all(|c| c.is_ascii_digit())
        && &run_id[8..9] == "_"
        && run_id[9..15].chars().all(|c| c.is_ascii_digit())
        && (&run_id[15..] == "_run" || &run_id[15..] == "_explore");
    if !valid {
        anyhow::bail!(
            "invalid RUN_ID: {:?}. RUN_ID must use the neutral pattern \
             YYYYMMDD_HHMMSS_run or YYYYMMDD_HHMMSS_explore. \
             Descriptive suffixes like _trade_mtl_tor leak task context \
             to the vision model and cause hallucination.",
            run_id
        );
    }
    Ok(())
}

fn send_to_daemon(socket_path: &str, script: &str) -> Result<()> {
    let mut stream = UnixStream::connect(socket_path)
        .with_context(|| format!("failed to connect to daemon at {socket_path}"))?;

    stream
        .write_all(script.as_bytes())
        .and_then(|_| stream.write_all(b"\n"))
        .with_context(|| "failed to send command to daemon")?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .with_context(|| "failed to read daemon response")?;

    print!("{response}");
    Ok(())
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}
