use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

pub trait Observer: Send + Sync {
    fn is_connected(&self) -> bool;
    fn detect_scene(&self) -> Scene;
    fn pixel_at(&self, x: u32, y: u32) -> Option<PixelColor>;
    fn template_match(&self, _name: &str) -> Option<(f64, f64)> {
        None
    }
    fn capture_screenshot(&self, _label: &str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!(
            "screenshots not supported by this observer"
        ))
    }
    fn capture_screenshot_flat(&self, _label: &str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!(
            "screenshots not supported by this observer"
        ))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub name: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct NullObserver;

impl Observer for NullObserver {
    fn is_connected(&self) -> bool {
        true
    }
    fn detect_scene(&self) -> Scene {
        Scene::default()
    }
    fn pixel_at(&self, _x: u32, _y: u32) -> Option<PixelColor> {
        None
    }
}

pub struct ScreenCaptureObserver {
    window_substring: String,
    run_id: Mutex<Option<String>>,
    run_dir: Mutex<Option<PathBuf>>,
    counter: AtomicU32,
    watch_path: Mutex<Option<PathBuf>>,
    json_log: Mutex<Option<Arc<Mutex<BufWriter<fs::File>>>>>,
}

impl ScreenCaptureObserver {
    pub fn new(window_substring: &str) -> Self {
        Self {
            window_substring: window_substring.to_string(),
            run_id: Mutex::new(None),
            run_dir: Mutex::new(None),
            counter: AtomicU32::new(0),
            watch_path: Mutex::new(None),
            json_log: Mutex::new(None),
        }
    }

    pub fn set_run_id(&self, run_id: &str) {
        *self.run_id.lock().unwrap() = Some(run_id.to_string());
    }

    pub fn set_watch(&self, path: PathBuf) {
        *self.watch_path.lock().unwrap() = Some(path);
    }

    pub fn set_json_log(&self, writer: Arc<Mutex<BufWriter<fs::File>>>) {
        *self.json_log.lock().unwrap() = Some(writer);
    }

    fn find_window(&self) -> Option<xcap::Window> {
        let all_windows = xcap::Window::all().ok()?;
        let sub_lower = self.window_substring.to_lowercase();
        let matches: Vec<&xcap::Window> = all_windows
            .iter()
            .filter(|w| {
                w.title()
                    .ok()
                    .is_some_and(|t| t.to_lowercase().contains(&sub_lower))
            })
            .collect();

        if matches.is_empty() {
            tracing::warn!(
                "no window found matching '{}'; available windows:",
                self.window_substring
            );
            for w in &all_windows {
                if let Ok(t) = w.title() {
                    tracing::info!("  \"{t}\"");
                }
            }
            return None;
        }

        if matches.len() > 1 {
            let titles: Vec<String> = matches.iter().filter_map(|w| w.title().ok()).collect();
            tracing::warn!(
                "{} windows match '{}', using first: {:?}",
                matches.len(),
                self.window_substring,
                titles,
            );
        }

        let win = matches[0];
        if let Ok(t) = win.title() {
            tracing::info!(
                "found window \"{t}\" matching substring \"{}\"",
                self.window_substring
            );
        }
        Some(win.clone())
    }

    fn ensure_run_dir(&self) -> anyhow::Result<PathBuf> {
        let mut dir = self.run_dir.lock().unwrap();
        if dir.is_none() {
            let dir_name = if let Some(ref id) = *self.run_id.lock().unwrap() {
                id.clone()
            } else {
                let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
                format!("{}_run", ts)
            };
            let path = PathBuf::from("screenshots").join(&dir_name);
            fs::create_dir_all(&path)?;
            tracing::info!("created screenshot directory: {}", path.display());
            *dir = Some(path);
        }
        Ok(dir.as_ref().unwrap().clone())
    }

    fn capture_and_save(&self, label: &str, flat: bool) -> anyhow::Result<String> {
        let window = self.find_window().ok_or_else(|| {
            anyhow::anyhow!(
                "no window found matching substring '{}'",
                self.window_substring
            )
        })?;
        let img = window.capture_image()?;

        let run_dir = if !flat {
            Some(self.ensure_run_dir()?)
        } else {
            None
        };

        let c = self.counter.fetch_add(1, Ordering::SeqCst) + 1;

        let path = if flat {
            let name = if label.is_empty() {
                format!("{:03}.png", c)
            } else {
                format!("{}.png", label)
            };
            let p = PathBuf::from("screenshots").join(&name);
            let _ = fs::create_dir_all("screenshots");
            p
        } else {
            let run_dir = run_dir.unwrap();
            let name = if label.is_empty() {
                format!("{:03}.png", c)
            } else {
                format!("{:03}_{}.png", c, label)
            };
            run_dir.join(&name)
        };

        img.save(&path)?;

        tracing::info!("screenshot saved: {}", path.display());

        if let Some(ref json_log) = *self.json_log.lock().unwrap() {
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let path_str = path.to_string_lossy().to_string();
            if let Ok(event) = serde_json::to_vec(&serde_json::json!({
                "ts": ts,
                "event": "screenshot",
                "path": path_str,
            })) {
                if let Ok(mut writer) = json_log.lock() {
                    let _ = writer
                        .write_all(&event)
                        .and_then(|_| writer.write_all(b"\n"))
                        .and_then(|_| writer.flush());
                }
            }
        }

        if let Some(ref watch) = *self.watch_path.lock().unwrap() {
            if let Err(e) = img.save(watch) {
                tracing::error!("failed to update watch screenshot: {e}");
            }
        }

        Ok(path.to_string_lossy().to_string())
    }
}

impl Observer for ScreenCaptureObserver {
    fn is_connected(&self) -> bool {
        self.find_window()
            .is_some_and(|w| !w.is_minimized().unwrap_or(false))
    }

    fn detect_scene(&self) -> Scene {
        Scene::default()
    }

    fn pixel_at(&self, x: u32, y: u32) -> Option<PixelColor> {
        let window = self.find_window()?;
        let img = window.capture_image().ok()?;
        let pixel = img.get_pixel(x, y);
        Some(PixelColor {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
        })
    }

    fn capture_screenshot(&self, label: &str) -> anyhow::Result<String> {
        self.capture_and_save(label, false)
    }

    fn capture_screenshot_flat(&self, label: &str) -> anyhow::Result<String> {
        self.capture_and_save(label, true)
    }
}
