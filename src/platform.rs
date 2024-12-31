#[cfg(not(target_arch = "wasm32"))]
pub mod plt {
    pub const IS_WASM: bool = false;
    pub const UI_SCALE: f32 = 1.0;
    pub const FONT_SCALE: f32 = UI_SCALE / 1.2;
    pub const ASPECT_RATIO: f32 = 15.0 / 8.0;
}

#[cfg(target_arch = "wasm32")]
pub mod plt {
    pub const IS_WASM: bool = true;
    pub const UI_SCALE: f32 = 0.8;
    pub const FONT_SCALE: f32 = UI_SCALE / 1.2;
    pub const ASPECT_RATIO: f32 = 16.0 / 10.0;
}

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
