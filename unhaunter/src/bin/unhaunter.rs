use unhaunter::app_run;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = unhaunter::assetidx_updater::update_assetidx_files() {
            eprintln!("Failed to update assetidx files: {}", e);
        }
    }
    app_run();
}
