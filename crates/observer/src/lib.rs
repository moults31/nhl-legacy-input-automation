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
    run_dir: Mutex<Option<PathBuf>>,
    counter: Mutex<u32>,
}

impl ScreenCaptureObserver {
    pub fn new(window_substring: &str) -> Self {
        Self {
            window_substring: window_substring.to_string(),
            run_dir: Mutex::new(None),
            counter: Mutex::new(0),
        }
    }

    fn find_window(&self) -> Option<xcap::Window> {
        xcap::Window::all().ok()?.into_iter().find(|w| {
            w.title().ok().is_some_and(|t| {
                t.to_lowercase()
                    .contains(&self.window_substring.to_lowercase())
            })
        })
    }

    fn ensure_run_dir(&self) -> anyhow::Result<PathBuf> {
        let mut dir = self.run_dir.lock().unwrap();
        if dir.is_none() {
            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let path = PathBuf::from("screenshots").join(format!("{}_run", ts));
            fs::create_dir_all(&path)?;
            tracing::info!("created screenshot directory: {}", path.display());
            *dir = Some(path);
        }
        Ok(dir.as_ref().unwrap().clone())
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
        let window = self.find_window().ok_or_else(|| {
            anyhow::anyhow!(
                "no window found matching substring '{}'",
                self.window_substring
            )
        })?;
        let img = window.capture_image()?;
        let run_dir = self.ensure_run_dir()?;
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        let filename = if label.is_empty() {
            format!("{:03}.png", counter)
        } else {
            format!("{:03}_{}.png", counter, label)
        };
        let path = run_dir.join(&filename);
        img.save(&path)?;
        tracing::info!("screenshot saved: {}", path.display());
        Ok(path.to_string_lossy().to_string())
    }
}
