use once_cell::sync::Lazy;

pub struct Config {
    pub window_sidebar_width: f32,
    pub window_main_width: f32,
    pub window_main_height: f32,
    pub selection_stroke: egui::Stroke,
    pub selection_rounding: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            window_sidebar_width: 272., 
            window_main_width: 448., 
            window_main_height: 480., 
            selection_stroke: egui::Stroke {
                width: 1.5,
                color: egui::Color32::from_rgb(0, 179, 134),
            },
            selection_rounding: 2.0,
        }
    }
}

impl Config {
    pub fn window_min(&self) -> egui::Vec2 {
        egui::Vec2 {
            x: self.window_main_width + self.window_sidebar_width,
            y: self.window_main_height,
        }
    }
}

// TODO: One day this could load from a TOML file
pub static CONFIG: Lazy<Config> = Lazy::new(Config::default);