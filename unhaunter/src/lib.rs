pub mod app;
pub mod report_timer;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn wasm_load() {
    app_run();
}

pub fn app_run() {
    app::app_run();
}
