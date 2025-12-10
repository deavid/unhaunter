// Module for manifest data structures

//! Manifest data structure for tracking generated voice lines.

use serde::{Deserialize, Serialize};
use unwalkie_types::WalkieTag;

/// Represents an entry in the `manifest.ron` file.
/// This struct stores all relevant metadata for a single generated voice line.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalkieLineManifestEntry {
    // Identification
    /// The name of the RON file from which this line originated (e.g., "low_visibility.ron").
    pub ron_file_source: String,
    /// The conceptual ID of the voice line (e.g., "DarkTorchReminder"), derived from `WalkieEventConceptEntry.name`.
    pub conceptual_id: String,
    /// The index of this line within the `lines` array of its parent `WalkieEventConceptEntry`.
    pub line_index: usize,

    // Source data
    /// The original TTS text.
    pub tts_text: String,
    /// The original subtitle text.
    pub subtitle_text: String,
    /// Tags associated with the line. Stored as a `Vec` in the manifest for stable order,
    /// which can be useful for consistent code generation.
    pub tags: Vec<WalkieTag>,

    // Generation artifacts & metadata
    /// The path to the generated OGG audio file, relative to `GENERATED_ASSETS_DIR`
    /// (e.g., "low_visibility_dark_torch_reminder_01.ogg").
    pub ogg_path: String,
    /// The duration of the generated audio file in seconds.
    pub length_seconds: u32,
    /// A SHA256 hash of the `generate_walkie_voice.sh` script used to generate this line.
    /// This helps detect if the generation script has changed, necessitating regeneration.
    pub generation_script_hash: String,
    /// A combined SHA256 hash of the `tts_text` and `generation_script_hash`.
    /// This signature is the primary mechanism for detecting if a voice line needs regeneration.
    pub combined_signature: String,
}
