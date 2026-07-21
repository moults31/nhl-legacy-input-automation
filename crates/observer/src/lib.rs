use std::ffi::CString;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tesseract::plumbing::TessBaseApi;

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
    fn ocr_analyze(&self) -> anyhow::Result<(OcrResult, Option<usize>)> {
        Err(anyhow::anyhow!("OCR not available"))
    }
    fn ocr_analyze_from_path(&self, _path: &str) -> anyhow::Result<(OcrResult, Option<usize>)> {
        Err(anyhow::anyhow!("OCR not available"))
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

    fn get_or_init_tesseract() -> anyhow::Result<&'static Mutex<TessBaseApi>> {
        static ENGINE: OnceLock<anyhow::Result<Mutex<TessBaseApi>>> = OnceLock::new();
        match ENGINE.get_or_init(|| {
            (|| -> anyhow::Result<Mutex<TessBaseApi>> {
                let mut api = TessBaseApi::create();
                let lang = CString::new("eng").unwrap();
                api.init_2(None, Some(&lang))
                    .map_err(|e| anyhow::anyhow!("Tesseract init_2 failed: {e:?}"))?;
                Ok(Mutex::new(api))
            })()
        }) {
            Ok(engine) => Ok(engine),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }

    fn ocr_image(
        &self,
        rgb: &image::RgbImage,
        w: u32,
        h: u32,
    ) -> anyhow::Result<(OcrResult, Option<usize>)> {
        let api_lock = Self::get_or_init_tesseract()?;
        let mut api = api_lock.lock().unwrap();

        let bpl = (w * 3) as i32;
        api.set_image(rgb.as_raw(), w as i32, h as i32, 3, bpl)
            .map_err(|e| anyhow::anyhow!("Tesseract set_image failed: {e:?}"))?;

        api.set_page_seg_mode(3);

        api.recognize()
            .map_err(|e| anyhow::anyhow!("Tesseract recognize failed: {e:?}"))?;

        let tsv_text = api
            .get_tsv_text(0)
            .map_err(|e| anyhow::anyhow!("Tesseract get_tsv_text failed: {e:?}"))?;

        let tsv_string = tsv_text.as_ref().to_string_lossy().into_owned();
        Ok(Self::parse_tsv_output(&tsv_string, rgb, w, h))
    }

    fn parse_tsv_output(
        tsv: &str,
        rgb: &image::RgbImage,
        w: u32,
        h: u32,
    ) -> (OcrResult, Option<usize>) {
        let mut result_lines: Vec<OcrLine> = Vec::new();
        let mut current_words: Vec<(i32, i32, i32, i32, String)> = Vec::new();
        let mut prev_line_key: Option<(i32, i32, i32)> = None;

        for line in tsv.lines().skip(1) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() < 12 {
                continue;
            }

            let level: i32 = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if level != 5 {
                continue;
            }

            let block: i32 = parts[2].parse().unwrap_or(0);
            let par: i32 = parts[3].parse().unwrap_or(0);
            let line_num: i32 = parts[4].parse().unwrap_or(0);
            let key = (block, par, line_num);

            if let Some(prev) = prev_line_key {
                if prev != key {
                    Self::flush_line(&mut result_lines, &mut current_words);
                }
            }

            let left: i32 = parts[6].parse().unwrap_or(0);
            let top: i32 = parts[7].parse().unwrap_or(0);
            let width: i32 = parts[8].parse().unwrap_or(0);
            let height: i32 = parts[9].parse().unwrap_or(0);
            let text = parts[11..].join("\t");

            if text.trim().is_empty() {
                continue;
            }

            current_words.push((left, top, left + width, top + height, text));
            prev_line_key = Some(key);
        }

        Self::flush_line(&mut result_lines, &mut current_words);

        let mut all_text = String::new();
        for line in &result_lines {
            if !all_text.is_empty() {
                all_text.push('\n');
            }
            all_text.push_str(&line.text);
        }

        let selected_index = Self::find_selected_by_luminance(rgb, &result_lines, w, h);

        (
            OcrResult {
                lines: result_lines,
                all_text,
                selected_index,
            },
            selected_index,
        )
    }

    fn flush_line(lines: &mut Vec<OcrLine>, words: &mut Vec<(i32, i32, i32, i32, String)>) {
        if words.is_empty() {
            return;
        }

        let line_text: Vec<&str> = words.iter().map(|(_, _, _, _, t)| t.as_str()).collect();
        let line_text = line_text.join(" ");

        let min_left = words.iter().map(|(l, _, _, _, _)| *l).min().unwrap_or(0);
        let min_top = words.iter().map(|(_, t, _, _, _)| *t).min().unwrap_or(0);
        let max_right = words.iter().map(|(_, _, r, _, _)| *r).max().unwrap_or(0);
        let max_bottom = words.iter().map(|(_, _, _, b, _)| *b).max().unwrap_or(0);

        let ocr_words: Vec<OcrWord> = words
            .iter()
            .map(|(l, t, r, b, txt)| OcrWord {
                text: txt.clone(),
                rect: OcrRect {
                    left: *l,
                    top: *t,
                    right: *r,
                    bottom: *b,
                },
            })
            .collect();

        lines.push(OcrLine {
            text: line_text,
            rect: OcrRect {
                left: min_left,
                top: min_top,
                right: max_right,
                bottom: max_bottom,
            },
            words: ocr_words,
        });

        words.clear();
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

    fn ocr_analyze(&self) -> anyhow::Result<(OcrResult, Option<usize>)> {
        let window = self.find_window().ok_or_else(|| {
            anyhow::anyhow!(
                "no window found matching substring '{}'",
                self.window_substring
            )
        })?;
        let img = window
            .capture_image()
            .map_err(|e| anyhow::anyhow!("OCR: window capture failed: {e}"))?;
        let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
        let (w, h) = rgb.dimensions();
        self.ocr_image(&rgb, w, h)
    }

    fn ocr_analyze_from_path(&self, path: &str) -> anyhow::Result<(OcrResult, Option<usize>)> {
        let img = image::open(path)
            .map_err(|e| anyhow::anyhow!("OCR: failed to open image {path}: {e}"))?
            .to_rgb8();
        let (w, h) = img.dimensions();
        self.ocr_image(&img, w, h)
    }
}
