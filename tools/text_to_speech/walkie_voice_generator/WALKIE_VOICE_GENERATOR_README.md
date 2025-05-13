# Walkie Voice Generator Tool (`walkie_voice_generator`)

This document outlines the usage of the `walkie_voice_generator` CLI tool, a crucial component of the Unhaunter game project. This Rust-based tool automates the pipeline for creating in-game walkie-talkie voice lines, from text definitions to game-ready audio assets and corresponding Rust code.

## Overview

The `walkie_voice_generator` streamlines the process of adding and managing voice lines for the Walkie-Talkie Buddy NPC. The primary workflow involves:

1.  **Defining Voice Lines:** Voice lines, their spoken text, subtitle text, and descriptive tags are defined in `.ron` (Rusty Object Notation) files.
2.  **Running the Tool:** Executing `walkie_voice_generator` triggers the following automated processes:
    *   **Parsing:** Reads all `.ron` definition files.
    *   **Audio Generation:** For new or modified lines, it invokes an external shell script (`generate_walkie_voice.sh`) which uses a Text-To-Speech (TTS) engine (Kokoro TTS) to synthesize speech into a temporary WAV file.
    *   **Audio Processing:** The same script then uses FFmpeg to apply walkie-talkie audio effects (filtering, noise, reverb), convert to mono, resample (11025Hz), and encode to a low-bitrate OGG Vorbis format (`-ab 32k`).
    *   **Metadata Extraction:** Invokes another script (`get_audio_duration.sh`) using `ffprobe` to determine the length in seconds of each generated OGG file.
    *   **Manifest Management:** Creates or updates a `manifest.ron` file. This manifest tracks all generated OGG files, their source definitions, audio duration, content signatures (for change detection), and associated metadata like tags and subtitles.
    *   **Rust Code Generation:** Generates `.rs` source files within the `unwalkie` game crate. This code provides type-safe enums and functions for the game to easily access and utilize the voice lines and their metadata (subtitles, tags, duration).

## Prerequisites

To use the `walkie_voice_generator` tool, the following must be installed and accessible in your system's PATH:

*   **Rust Toolchain:** For compiling and running the tool itself.
*   **Kokoro TTS CLI:** The command-line interface for the Kokoro Text-To-Speech engine.
*   **FFmpeg & ffprobe:** The FFmpeg suite, including `ffmpeg` (for audio processing) and `ffprobe` (for metadata extraction like audio duration).
*   **Bash (or compatible shell):** For executing the helper scripts.

The tool also expects specific helper scripts to be present:
*   `tools/text_to_speech/scripts/generate_walkie_voice.sh`
*   `tools/text_to_speech/scripts/get_audio_duration.sh`

## Directory Structure

The tool interacts with a predefined directory structure within the Unhaunter project:

*   **Tool Source:** `tools/text_to_speech/walkie_voice_generator/`
*   **Input RON Definitions:** `tools/text_to_speech/walkie_phrases/` (non-recursive scan)
*   **Helper Scripts:** `tools/text_to_speech/scripts/`
*   **Temporary Audio:** `temp_audio/` (for intermediate WAV files; created and cleaned by the tool)
*   **Output OGG Audio:** `assets/walkie/generated/`
*   **Output Manifest:** `assets/walkie/generated/manifest.ron`
*   **Output Rust Code:** `unwalkie/src/generated/`
*   **Shared Types Crate:** `unwalkie_types/` (defines `WalkieTag` and `VoiceLineData` used by both the tool and the game)

## RON File Format (`*.ron`)

Voice lines are defined in `.ron` files located in `tools/text_to_speech/walkie_phrases/`. Each file can group multiple "concepts," where each concept can have several voice line variations.

**Example (`tools/text_to_speech/walkie_phrases/sample_lines.ron`):**
```ron
o_speech/walkie_phrases/sample_lines.ron
(  // is a WalkiePhraseFile struct
  event_lines: [
    (  // is a WalkieEventConceptEntry
      name: "PlayerLowOnSanity", // PascalCase; forms part of Rust enum & filenames
      lines: [  // is a Vec<WalkieLineEntry>
        (
          tts_text: "You're sounding a bit rough there, sure you're alright?",
          subtitle_text: "Sounding a bit rough there. You alright?",
          tags: {ConcernedWarning, PlayerStruggling, MediumLength}, // Set of WalkieTag
        ),
        (
          tts_text: "Hey, if you need a breather, the van's always there. Just sayin'.",
          subtitle_text: "Van's a safe spot if you need a breather.",
          tags: {Encouraging, ReminderLow, MediumLength},
        ),
      ],
    ),
    (
      name: "FlashlightOffInDark",
      lines: [
        (
          tts_text: "Bit dark to be wandering about without your torch, isn't it?",
          subtitle_text: "Dark in here. Got your torch?",
          tags: {SlightlyImpatient, SnarkyHumor, ShortBrevity, FirstTimeHint},
        ),
      ],
    ),
  ],
)
```

*   **`WalkiePhraseFile`:** The root struct. Contains `event_lines`.
*   **`WalkieEventConceptEntry`:**
    *   `name: String`: A unique (within the file, and ideally globally for clarity) PascalCase identifier for the voice line concept (e.g., "FlashlightOffInDark").
    *   `lines: Vec<WalkieLineEntry>`: A list of variations for this concept.
*   **`WalkieLineEntry`:**
    *   `tts_text: String`: The exact text for Kokoro TTS.
    *   `subtitle_text: String`: The text for in-game subtitles.
    *   `tags: HashSet<WalkieTag>`: A set of tags (defined in `unwalkie_types::WalkieTag`) like `{SnarkyHumor, ShortBrevity}`.

## `manifest.ron` File

Located at `assets/walkie/generated/manifest.ron`, this file is automatically managed by the tool. It's a RON representation of a `HashMap<String, WalkieLineManifestEntry>`, where the key is a unique identifier for each line variant (e.g., "sample_lines/PlayerLowOnSanity/0").

Each `WalkieLineManifestEntry` stores:
*   Source RON file, conceptual name, line index.
*   `tts_text`, `subtitle_text`, sorted `Vec<WalkieTag>`.
*   Relative `ogg_path` (e.g., "sample_lines_player_low_on_sanity_01.ogg").
*   `length_seconds: u32`.
*   `generation_script_hash` (hash of `generate_walkie_voice.sh`).
*   `combined_signature` (hash of `tts_text` + `generation_script_hash`) for change detection.

## Generated Rust Code

The tool generates Rust modules in `unwalkie/src/generated/`.

*   **`unwalkie/src/generated/mod.rs`:**
    *   Re-exports `WalkieTag` from `unwalkie_types`.
    *   Declares `pub mod <ron_filename_snake_case>;` for each input RON file.
*   **`unwalkie/src/generated/<ron_filename_snake_case>.rs` (e.g., `sample_lines.rs`):**
    *   Imports `VoiceLineData` and `WalkieTag` from `unwalkie_types`.
    *   Defines a public `concept` module (e.g., `pub mod concept`).
    *   Inside `concept`, defines an enum named after the RON file (e.g., `pub enum SampleLines { PlayerLowOnSanity, FlashlightOffInDark }`). The variants match the `name` fields from the RON.
    *   Provides a public function: `pub fn get_lines(concept_type: concept::SampleLines) -> &'static [VoiceLineData]`.
    *   This function returns a static slice of `VoiceLineData` structs, one for each line variation under that concept, populated with the OGG path, subtitle, tags, and duration.

The `VoiceLineData` struct itself is defined in the `unwalkie_types` crate.

## Using the Tool

Run the tool from the Unhaunter project's root directory.

### 1. Generate a Sample RON File (Optional)
To see an example of the input RON structure:
```bash
cargo run --bin walkie_voice_generator -- --generate-sample-ron > tools/text_to_speech/walkie_phrases/new_lines.ron
```
Edit `new_lines.ron` with your actual voice line definitions.

### 2. Process All Definitions & Generate Assets
This is the main command to run after defining or modifying your `.ron` files:
```bash
cargo run --bin walkie_voice_generator
```
This command will:
1.  Scan `tools/text_to_speech/walkie_phrases/` for `.ron` files.
2.  Load `assets/walkie/generated/manifest.ron` (or create it).
3.  For each voice line:
    *   Calculate its signature.
    *   If new, changed, or forced, it calls `generate_walkie_voice.sh` (which uses Kokoro and FFmpeg) to produce the OGG in `assets/walkie/generated/` (e.g., `sample_lines_player_low_on_sanity_01.ogg`).
    *   Calls `get_audio_duration.sh` (using `ffprobe`) to get the OGG length.
    *   Updates the line's entry in the manifest.
4.  Saves the updated manifest.
5.  Generates/overwrites the Rust code in `unwalkie/src/generated/`.
6.  Warns about any OGG files in `assets/walkie/generated/` not listed in the manifest (unless `--delete-unused` is used).

### Command-Line Options

*   `--generate-sample-ron`: Prints a sample RON file structure to stdout and exits.
*   `--force-regenerate [PATTERN]`: Forces audio regeneration.
    *   No `[PATTERN]`: Regenerates all lines.
    *   `[PATTERN]`: Regenerates lines where the `name` (conceptual ID) in RON contains the pattern (case-sensitive). Supports `*` as a wildcard at the end for prefix matching (e.g., `Player*`).
*   `--delete-unused`: Deletes orphaned OGG files from `assets/walkie/generated/` that are no longer in the manifest. **Use with caution.** By default, the tool only warns.
*   `--parallel-jobs <NUMBER>`: Number of threads for parallel audio generation (default: 1). Using more jobs can significantly speed up processing for many lines but increases system load.

## Important Notes

*   **External Scripts & Tools**: The correct functioning of `Kokoro TTS`, `ffmpeg`, `ffprobe`, and the provided `.sh` scripts is critical. Ensure they are executable and correctly configured.
*   **Paths**: All key directory paths are hardcoded in the tool (see `constants.rs`).
*   **Error Handling**: The tool will panic and exit on critical errors (e.g., script failures, missing essential directories, parsing errors), providing error messages to `stderr`.

This README provides a comprehensive guide to using the `walkie_voice_generator`. For precise details on data structures or specific logic, refer to the tool's source code and the project's main design documents.
