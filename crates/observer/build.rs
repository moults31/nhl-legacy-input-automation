use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DETECTION_URL: &str =
    "https://huggingface.co/robertknight/ocrs/resolve/main/text-detection-ssfbcj81.rten";
const RECOGNITION_URL: &str =
    "https://huggingface.co/robertknight/ocrs/resolve/main/text-rec-checkpoint-s52qdbqt.rten";

fn download(url: &str, dest: &Path) {
    let label = dest.file_name().unwrap().to_string_lossy();
    eprintln!("downloading {label} from {url}...");

    let response = reqwest::blocking::get(url).unwrap_or_else(|e| {
        panic!(
            "Failed to download OCR model from {url}: {e}\n\
             Disable the 'ocr-models' feature if you don't need OCR: \
             cargo build --no-default-features"
        );
    });

    if !response.status().is_success() {
        panic!(
            "HTTP {} while downloading OCR model from {url}\n\
             Disable the 'ocr-models' feature if you don't need OCR: \
             cargo build --no-default-features",
            response.status()
        );
    }

    let bytes = response
        .bytes()
        .unwrap_or_else(|e| panic!("Failed to read OCR model body from {url}: {e}"));

    fs::write(dest, &bytes)
        .unwrap_or_else(|e| panic!("Failed to write OCR model to {}: {e}", dest.display()));

    eprintln!(
        "  -> {} ({:.1} MB)",
        dest.display(),
        bytes.len() as f64 / 1_048_576.0
    );
}

fn main() {
    if env::var("CARGO_FEATURE_OCR_MODELS").is_err() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let out_det = out_dir.join("text-detection.rten");
    let out_rec = out_dir.join("text-recognition.rten");

    if !out_det.exists() {
        download(DETECTION_URL, &out_det);
    }
    if !out_rec.exists() {
        download(RECOGNITION_URL, &out_rec);
    }
}
