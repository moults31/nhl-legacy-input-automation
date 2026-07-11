use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use ocrs::{ImageSource, OcrEngine, OcrEngineParams, TextItem};

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
    fn ocr_analyze(&self) -> Option<(OcrResult, Option<usize>)> {
        None
    }
    fn ocr_analyze_from_path(&self, _path: &str) -> Option<(OcrResult, Option<usize>)> {
        None
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct OcrRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl OcrRect {
    pub fn center(&self) -> (i32, i32) {
        ((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OcrWord {
    pub text: String,
    pub rect: OcrRect,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OcrLine {
    pub text: String,
    pub rect: OcrRect,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<OcrWord>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OcrResult {
    pub lines: Vec<OcrLine>,
    pub all_text: String,
    #[serde(skip_serializing)]
    pub selected_index: Option<usize>,
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
    ocr_engine: Mutex<Option<OcrEngine>>,
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
            ocr_engine: Mutex::new(None),
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

    pub fn counter(&self) -> u32 {
        self.counter.load(Ordering::SeqCst)
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

    fn extract_ocr(
        engine: &OcrEngine,
        rgb: &image::RgbImage,
        (w, h): (u32, u32),
    ) -> Option<(OcrResult, Option<usize>)> {
        let img_source = ImageSource::from_bytes(rgb.as_raw(), (w, h)).ok()?;
        let input = engine.prepare_input(img_source).ok()?;
        let word_rects = engine.detect_words(&input).ok()?;
        let text_lines = engine.find_text_lines(&input, &word_rects);
        let recognized = engine.recognize_text(&input, &text_lines).ok()?;

        let mut result_lines = Vec::new();
        let mut all_text = String::new();

        for line_opt in recognized.iter().flatten() {
            let rect = line_opt.bounding_rect();
            let text = line_opt.to_string();
            if text.trim().is_empty() {
                continue;
            }

            let ocr_rect = OcrRect {
                left: rect.left(),
                top: rect.top(),
                right: rect.right(),
                bottom: rect.bottom(),
            };

            let words: Vec<OcrWord> = line_opt
                .words()
                .map(|w| {
                    let wr = w.bounding_rect();
                    OcrWord {
                        text: w.to_string(),
                        rect: OcrRect {
                            left: wr.left(),
                            top: wr.top(),
                            right: wr.right(),
                            bottom: wr.bottom(),
                        },
                    }
                })
                .collect();

            if !all_text.is_empty() {
                all_text.push('\n');
            }
            all_text.push_str(&text);

            result_lines.push(OcrLine {
                text,
                rect: ocr_rect,
                words,
            });
        }

        let selected_index = Self::find_selected_by_luminance(rgb, &result_lines, w, h);

        Some((
            OcrResult {
                lines: result_lines,
                all_text,
                selected_index,
            },
            selected_index,
        ))
    }

    fn find_selected_by_luminance(
        rgb: &image::RgbImage,
        lines: &[OcrLine],
        img_w: u32,
        img_h: u32,
    ) -> Option<usize> {
        let mut best_idx = None;
        let mut best_lum = 0.0;

        for (i, line) in lines.iter().enumerate() {
            let (cx, cy) = line.rect.center();
            if cx < 0 || cy < 0 || cx as u32 >= img_w || cy as u32 >= img_h {
                continue;
            }
            let pixel = rgb.get_pixel(cx as u32, cy as u32);
            let lum = 0.299_f64 * f64::from(pixel[0])
                + 0.587_f64 * f64::from(pixel[1])
                + 0.114_f64 * f64::from(pixel[2]);
            if lum > best_lum {
                best_lum = lum;
                best_idx = Some(i);
            }
        }

        best_idx
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

    fn ocr_analyze(&self) -> Option<(OcrResult, Option<usize>)> {
        let window = self.find_window()?;
        let img = window.capture_image().ok()?;
        let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
        let (w, h) = rgb.dimensions();

        let mut guard = self.ocr_engine.lock().ok()?;
        if guard.is_none() {
            *guard = Some(OcrEngine::new(OcrEngineParams::default()).ok()?);
        }
        let engine = guard.as_ref().unwrap();

        Self::extract_ocr(engine, &rgb, (w, h))
    }

    fn ocr_analyze_from_path(&self, path: &str) -> Option<(OcrResult, Option<usize>)> {
        let img = image::open(path).ok()?.to_rgb8();
        let (w, h) = img.dimensions();

        let mut guard = self.ocr_engine.lock().ok()?;
        if guard.is_none() {
            *guard = Some(OcrEngine::new(OcrEngineParams::default()).ok()?);
        }
        let engine = guard.as_ref().unwrap();

        Self::extract_ocr(engine, &img, (w, h))
    }
}
