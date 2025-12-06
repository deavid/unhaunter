#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
pub fn find_assets_directory() -> Option<PathBuf> {
    // 1. Check for CARGO_MANIFEST_DIR (development mode)

    use std::{env, path::PathBuf};
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let assets_path = PathBuf::from(manifest_dir).join("assets");
        if assets_path.is_dir() {
            return Some(assets_path);
        }
    }

    // 2. If not found, assume assets are alongside the executable (release mode)
    if let Ok(executable_path) = env::current_exe()
        && let Some(executable_dir) = executable_path.parent()
    {
        let assets_path = executable_dir.join("assets");
        if assets_path.is_dir() {
            return Some(assets_path);
        }
    }

    // 3. If still not found, return None
    None
}
