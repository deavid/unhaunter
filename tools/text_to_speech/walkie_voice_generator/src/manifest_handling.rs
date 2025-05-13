//! Module for manifest loading and saving logic
//!
//! Functions for loading and saving the manifest file.

use crate::constants::{GENERATED_ASSETS_DIR, MANIFEST_FILENAME};
use crate::manifest_types::WalkieLineManifestEntry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Loads the manifest file (`manifest.ron`) from the `GENERATED_ASSETS_DIR`.
///
/// # Returns
/// A `Result` containing a `HashMap` of manifest entries (keyed by a unique string identifier)
/// if successful, or an `anyhow::Error` if the manifest cannot be read or parsed.
/// If the manifest file does not exist, an empty `HashMap` is returned.
pub fn load_manifest() -> Result<HashMap<String, WalkieLineManifestEntry>, anyhow::Error> {
    let manifest_path = Path::new(GENERATED_ASSETS_DIR).join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        // If no manifest exists, it's not an error; just start with an empty one.
        return Ok(HashMap::new());
    }

    // Open and read the manifest file.
    let mut file = File::open(manifest_path)
        .map_err(|e| anyhow::anyhow!("Failed to open manifest file: {}", e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| anyhow::anyhow!("Failed to read manifest file: {}", e))?;

    // Deserialize the RON content into the manifest HashMap.
    let manifest: HashMap<String, WalkieLineManifestEntry> = ron::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse manifest RON: {}", e))?;
    Ok(manifest)
}

/// Saves the provided manifest `HashMap` to the `manifest.ron` file in `GENERATED_ASSETS_DIR`.
/// The manifest is pretty-printed for readability.
///
/// # Arguments
/// * `manifest` - A reference to the `HashMap` containing the manifest entries to save.
///
/// # Returns
/// An `Ok(())` if successful, or an `anyhow::Error` if the manifest cannot be serialized or written.
pub fn save_manifest(
    manifest: &HashMap<String, WalkieLineManifestEntry>,
) -> Result<(), anyhow::Error> {
    let manifest_path = Path::new(GENERATED_ASSETS_DIR).join(MANIFEST_FILENAME);

    // Configure RON pretty printing.
    let pretty_config = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(false); // Changed to false to remove array enumeration

    // Serialize the manifest to a RON string.
    let serialized_manifest = ron::ser::to_string_pretty(manifest, pretty_config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize manifest to RON: {}", e))?;

    // Create/truncate and write to the manifest file.
    let mut file = File::create(manifest_path)
        .map_err(|e| anyhow::anyhow!("Failed to create/open manifest file for writing: {}", e))?;
    file.write_all(serialized_manifest.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to write to manifest file: {}", e))?;
    Ok(())
}
