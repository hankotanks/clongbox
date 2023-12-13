use once_cell::sync::Lazy;

pub struct Config {
    pub window_sidebar_width: f32,
    pub window_main_width: f32,
    pub window_main_height: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            window_sidebar_width: 260., 
            window_main_width: 460., 
            window_main_height: 480., 
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
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::default());