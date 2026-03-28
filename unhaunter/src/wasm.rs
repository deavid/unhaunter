use crate::app;
use crate::app_args::AppArgs;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub(crate) fn wasm_load() {
    app_run(AppArgs::default_wasm()); // Use default for WASM
}

pub fn app_run(args: AppArgs) {
    app::app_run(args);
}
