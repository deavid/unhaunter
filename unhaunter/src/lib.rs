pub mod app;
pub mod assetidx_updater;
pub mod report_timer;
pub mod utils;

use uncore::resources::cli_options::CliOptions;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn wasm_load() {
    app_run(CliOptions::default()); // Use default for WASM
}

pub fn app_run(cli_options: CliOptions) {
    app::app_run(cli_options);
}
