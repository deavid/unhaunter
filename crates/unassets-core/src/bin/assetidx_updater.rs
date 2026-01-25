use anyhow::Result;

fn main() -> Result<()> {
    println!("Scanning for assets...");
    unassets_core::assets::index_updater::update_assetidx_files()
}
