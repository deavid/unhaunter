// Module for constants

//! Constants used throughout the walkie_voice_generator application.

/// Directory containing the input RON files with voice line definitions.
pub(crate) const WALKIE_PHRASES_DIR: &str = "crates/tools/text_to_speech/walkie_phrases/";
/// Directory where generated OGG audio assets will be stored.
pub(crate) const GENERATED_ASSETS_DIR: &str = "assets/walkie/generated/";
/// Directory where generated Rust code (for unwalkiecore crate) will be stored.
pub(crate) const GENERATED_RUST_DIR: &str = "crates/unwalkie-core/src/generated/";
/// Filename for the manifest that tracks generated audio files and their metadata.
pub(crate) const MANIFEST_FILENAME: &str = "manifest.ron";
/// Path to the shell script responsible for TTS and audio effects generation.
pub(crate) const GENERATE_SCRIPT_PATH: &str =
    "crates/tools/text_to_speech/scripts/generate_walkie_voice.sh";
/// Path to the shell script responsible for extracting audio duration.
pub(crate) const DURATION_SCRIPT_PATH: &str =
    "crates/tools/text_to_speech/scripts/get_audio_duration.sh";
/// Directory for storing temporary audio files (e.g., intermediate WAV files).
pub(crate) const TEMP_AUDIO_DIR: &str = "crates/tools/text_to_speech/temp_audio/";
