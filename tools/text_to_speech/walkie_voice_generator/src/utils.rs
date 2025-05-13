// Module for utility functions (hashing, file scanning, etc.)

//! Utility functions for the walkie_voice_generator tool.

use crate::ron_types::{WalkieEventConceptEntry, WalkieLineEntry, WalkiePhraseFile};
use ron::ser::{PrettyConfig, to_string_pretty};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use unwalkie_types::WalkieTag;
use walkdir::WalkDir;

/// Generates a sample RON file content and prints it to stdout.
/// This helps users understand the expected input format for voice line definitions.
pub fn generate_sample_ron() -> String {
    let sample_data = WalkiePhraseFile {
        event_lines: vec![
            WalkieEventConceptEntry {
                name: "PlayerSpottedGhost".to_string(),
                lines: vec![
                    WalkieLineEntry {
                        tts_text: "I think I saw something over there!".to_string(),
                        subtitle_text: "Saw something...".to_string(),
                        tags: vec![WalkieTag::ShortBrevity, WalkieTag::FirstTimeHint]
                            .into_iter()
                            .collect::<HashSet<WalkieTag>>(),
                    },
                    WalkieLineEntry {
                        tts_text: "Definitely a spooky presence nearby.".to_string(),
                        subtitle_text: "Spooky presence.".to_string(),
                        tags: vec![WalkieTag::ShortBrevity, WalkieTag::NeutralObservation]
                            .into_iter()
                            .collect::<HashSet<WalkieTag>>(),
                    },
                ],
            },
            WalkieEventConceptEntry {
                name: "LowSanityWarning".to_string(),
                lines: vec![
                    WalkieLineEntry {
                        tts_text: "My head is spinning, I need to get out.".to_string(),
                        subtitle_text: "Head spinning...".to_string(),
                        tags: vec![WalkieTag::PlayerStruggling, WalkieTag::ConcernedWarning]
                            .into_iter()
                            .collect::<HashSet<WalkieTag>>(),
                    },
                    WalkieLineEntry {
                        tts_text: "Can't take much more of this, feeling weak.".to_string(),
                        subtitle_text: "Feeling weak.".to_string(),
                        tags: vec![
                            WalkieTag::PlayerStruggling,
                            WalkieTag::LongDetailed,
                            WalkieTag::ConcernedWarning,
                        ]
                        .into_iter()
                        .collect::<HashSet<WalkieTag>>(),
                    },
                    WalkieLineEntry {
                        tts_text: "Just a routine check, everything seems quiet for now."
                            .to_string(),
                        subtitle_text: "All quiet.".to_string(),
                        tags: vec![WalkieTag::NeutralObservation, WalkieTag::MediumLength]
                            .into_iter()
                            .collect::<HashSet<WalkieTag>>(),
                    },
                ],
            },
            WalkieEventConceptEntry {
                name: "FirstTimeReminder".to_string(),
                lines: vec![WalkieLineEntry {
                    tts_text: "Remember to use your EMF reader to find disturbances.".to_string(),
                    subtitle_text: "Use EMF reader.".to_string(),
                    tags: vec![WalkieTag::FirstTimeHint, WalkieTag::ReminderLow]
                        .into_iter()
                        .collect::<HashSet<WalkieTag>>(),
                }],
            },
        ],
    };

    let pretty_config = PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        // .enumerate_arrays(true) // This line will be commented out or removed
        .struct_names(true); // Ensure struct names are enabled

    match to_string_pretty(&sample_data, pretty_config) {
        Ok(ron_string) => ron_string,
        Err(e) => format!(
            "// Failed to generate sample RON: {}\n// Using a fallback minimal example.\n(\n    event_lines: []\n)",
            e
        ),
    }
}

/// Calculates the SHA256 hash of a given script file.
/// This is used to detect changes in external scripts (e.g., `generate_walkie_voice.sh`)
/// which would necessitate regenerating audio files.
///
/// # Arguments
/// * `script_path` - The path to the script file to hash.
///
/// # Returns
/// A `Result` containing the hex-encoded SHA256 hash string if successful,
/// or an `anyhow::Error` if the file cannot be read or hashing fails.
pub fn calculate_script_hash(script_path: &str) -> Result<String, anyhow::Error> {
    let mut file = File::open(script_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to open script file {} for hashing: {}",
            script_path,
            e
        )
    })?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read script file {} for hashing: {}",
            script_path,
            e
        )
    })?;
    Ok(format!("{:x}", hasher.finalize())) // Format hash as a hex string.
}

/// Calculates a combined SHA256 signature for a voice line.
/// The signature is derived from the TTS text and the hash of the generation script.
/// This is the primary mechanism for determining if a voice line needs to be regenerated.
///
/// # Arguments
/// * `tts_text` - The text-to-speech content of the voice line.
/// * `script_hash` - The SHA256 hash of the generation script.
///
/// # Returns
/// A hex-encoded SHA256 hash string representing the combined signature.
pub fn calculate_combined_signature(tts_text: &str, script_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tts_text.as_bytes());
    hasher.update(script_hash.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Scans a directory for RON files (`.ron` extension).
/// It performs a non-recursive search (max_depth = 1).
/// If the specified directory does not exist, it prints an error and exits the program.
///
/// # Arguments
/// * `dir_path` - The path to the directory to scan.
///
/// # Returns
/// A `Result` containing a `Vec<PathBuf>` of found RON files if successful,
/// or an `anyhow::Error` if directory traversal fails (though critical errors like
/// non-existence lead to program exit).
pub fn scan_ron_files(dir_path: &str) -> Result<Vec<PathBuf>, anyhow::Error> {
    // Critical check: Ensure the source directory for RON files exists.
    // If not, the tool cannot proceed, so an error is printed and the program exits.
    if !Path::new(dir_path).exists() {
        eprintln!(
            "Error: RON source directory '{}' does not exist. Please create it and add RON files.",
            dir_path
        );
        std::process::exit(1);
    }

    let mut ron_files = Vec::new();
    // WalkDir is used to iterate over directory entries.
    // `max_depth(1)` ensures only files directly within `dir_path` are considered (not subdirectories).
    for entry in WalkDir::new(dir_path)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        // Check if the entry is a file and has a ".ron" extension.
        if path.is_file() && path.extension() == Some(OsStr::new("ron")) {
            // Check if the filename starts with an underscore
            if !path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .starts_with('_')
            {
                ron_files.push(path.to_path_buf());
            }
        }
    }
    Ok(ron_files)
}
