fn main() {
    if let Err(e) = unassetidx_updater::update_assetidx_files() {
        eprintln!("Failed to update assetidx files: {}", e);
        std::process::exit(1);
    }
}
