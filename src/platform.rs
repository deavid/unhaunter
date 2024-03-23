#[cfg(not(target_arch = "wasm32"))]
pub mod plt {
    pub const IS_WASM: bool = false;
    pub const UI_SCALE: f32 = 1.0;
}

#[cfg(target_arch = "wasm32")]
pub mod plt {
    pub const IS_WASM: bool = true;
    pub const UI_SCALE: f32 = 0.7;
}
