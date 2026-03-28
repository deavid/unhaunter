use crate::app;
use uncommon_app_core::cli::CliOptions;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub(crate) fn wasm_load() {
    app_run(CliOptions::default()); // Use default for WASM
}

pub fn app_run(cli_options: CliOptions) {
    app::app_run(cli_options);
}
