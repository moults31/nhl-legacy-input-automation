pub trait Observer: Send + Sync {
    fn is_connected(&self) -> bool;
    fn detect_scene(&self) -> Scene;
    fn pixel_at(&self, x: u32, y: u32) -> Option<PixelColor>;
    fn template_match(&self, _name: &str) -> Option<(f64, f64)> {
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
