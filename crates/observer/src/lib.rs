use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

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
    counter: Mutex<u32>,
    watch_path: Mutex<Option<PathBuf>>,
}

impl ScreenCaptureObserver {
    pub fn new(window_substring: &str) -> Self {
        Self {
            window_substring: window_substring.to_string(),
            run_id: Mutex::new(None),
            run_dir: Mutex::new(None),
            counter: Mutex::new(0),
            watch_path: Mutex::new(None),
        }
    }

    pub fn set_run_id(&self, run_id: &str) {
        *self.run_id.lock().unwrap() = Some(run_id.to_string());
    }

    pub fn set_watch(&self, path: PathBuf) {
        *self.watch_path.lock().unwrap() = Some(path);
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
            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let dir_name = if let Some(ref id) = *self.run_id.lock().unwrap() {
                format!("{}_{}", ts, id)
            } else {
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

        let mut counter = self.counter.lock().unwrap();
        *counter += 1;

        let path = if flat {
            let name = if label.is_empty() {
                format!("{:03}.png", counter)
            } else {
                format!("{}.png", label)
            };
            let p = PathBuf::from("screenshots").join(&name);
            let _ = fs::create_dir_all("screenshots");
            p
        } else {
            let run_dir = self.ensure_run_dir()?;
            let name = if label.is_empty() {
                format!("{:03}.png", counter)
            } else {
                format!("{:03}_{}.png", counter, label)
            };
            run_dir.join(&name)
        };

        img.save(&path)?;
        drop(counter);

        tracing::info!("screenshot saved: {}", path.display());

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
