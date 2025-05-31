// Module for processing RON files and generating audio

//! Functions for processing RON files, generating audio, and managing generated files.

use crate::constants::{
    DURATION_SCRIPT_PATH, GENERATED_ASSETS_DIR, GENERATE_SCRIPT_PATH, TEMP_AUDIO_DIR,
};
use crate::manifest_types::WalkieLineManifestEntry;
use crate::ron_types::{WalkieLineEntry, WalkiePhraseFile};
use crate::utils::calculate_combined_signature;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use unwalkie_types::WalkieTag;
use walkdir::WalkDir;

/// Represents a single audio generation task for a specific voice line.
#[derive(Debug, Clone)]
pub struct AudioGenerationTask {
    pub ron_filename_str: String,
    pub conceptual_id: String,
    pub line_idx: usize,
    pub line_entry: WalkieLineEntry, // Contains tts_text, subtitle_text, tags
    pub script_hash: String,
    pub force_regenerate_pattern: Option<String>,
    pub ron_file_sub_dir: String,            // New field: e.g., "base1"
    pub line_specific_filename_stem: String, // New field: e.g., "concept_line_01"
    pub ogg_path_relative_to_generated_dir: String, // e.g., "base1/concept_line_01.ogg"
    pub ogg_path_absolute: PathBuf,          // Full absolute path to the OGG file
    pub detailed_manifest_key: String,
}

/// Generates an audio file for a given TTS text and base filename.
/// It calls the `GENERATE_SCRIPT_PATH` shell script which handles TTS and ffmpeg processing.
///
/// # Arguments
/// * `tts_text` - The text to synthesize.
/// * `ron_file_sub_dir` - The subdirectory for the output files (e.g., "base1").
/// * `line_specific_filename_stem` - The base name for the output files (e.g., "concept_line_01"),
///   `.wav` and `.ogg` extensions will be appended.
///
/// # Returns
/// A `Result` containing a tuple of `(PathBuf, PathBuf)` for the temporary WAV path
/// and final OGG path respectively, or an `anyhow::Error` if script execution fails.
pub fn generate_audio_for_line(
    tts_text: &str,
    ron_file_sub_dir: &str,            // e.g., "base1"
    line_specific_filename_stem: &str, // e.g., "concept_line_01"
) -> Result<(PathBuf, PathBuf), anyhow::Error> {
    // Define paths for temporary WAV and final OGG files.
    // Temporary WAV can still be flat in TEMP_AUDIO_DIR for simplicity,
    // as it's deleted by the script.
    let temp_wav_filename = format!("{}.wav", line_specific_filename_stem);
    let temp_wav_path = Path::new(TEMP_AUDIO_DIR).join(temp_wav_filename);

    // Final OGG path will be in a subdirectory.
    let final_ogg_dir = Path::new(GENERATED_ASSETS_DIR).join(ron_file_sub_dir);
    let final_ogg_filename = format!("{}.ogg", line_specific_filename_stem);
    let final_ogg_path = final_ogg_dir.join(final_ogg_filename);

    // Ensure the target directory for the OGG file exists.
    fs::create_dir_all(&final_ogg_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create directory {}: {}",
            final_ogg_dir.display(),
            e
        )
    })?;

    // Ensure the generation script is executable. This is important on some systems or
    // if permissions were reset.
    let chmod_status = Command::new("chmod")
        .arg("+x")
        .arg(GENERATE_SCRIPT_PATH)
        .status()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute chmod on generation script {}: {}",
                GENERATE_SCRIPT_PATH,
                e
            )
        })?;
    if !chmod_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to make generation script {} executable (exit code: {:?})",
            GENERATE_SCRIPT_PATH,
            chmod_status.code()
        ));
    }

    // Execute the generation script.
    // Arguments: <tts_text> <temp_wav_path> <final_ogg_path>
    let output = Command::new(GENERATE_SCRIPT_PATH)
        .arg(tts_text)
        .arg(temp_wav_path.to_str().unwrap()) // Path to string conversion should be safe.
        .arg(final_ogg_path.to_str().unwrap())
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute generation script {}: {}",
                GENERATE_SCRIPT_PATH,
                e
            )
        })?;

    // Check if the script executed successfully.
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "{} script failed with status: {}\nStdout:\n{}\nStderr:\n{}",
            GENERATE_SCRIPT_PATH,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    // The script handles deletion of temp_wav_path on its own success.
    Ok((temp_wav_path, final_ogg_path))
}

/// Gets the duration of an OGG audio file in seconds.
/// It calls the `DURATION_SCRIPT_PATH` shell script which uses `ffprobe`.
///
/// # Arguments
/// * `ogg_path` - Path to the OGG file.
///
/// # Returns
/// A `Result` containing the duration as `u32` (rounded to the nearest second),
/// or an `anyhow::Error` if script execution or duration parsing fails.
pub fn get_audio_duration(ogg_path: &Path) -> Result<u32, anyhow::Error> {
    // Ensure the duration script is executable.
    let chmod_status = Command::new("chmod")
        .arg("+x")
        .arg(DURATION_SCRIPT_PATH)
        .status()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute chmod on duration script {}: {}",
                DURATION_SCRIPT_PATH,
                e
            )
        })?;
    if !chmod_status.success() {
        return Err(anyhow::anyhow!(
            "Failed to make duration script {} executable (exit code: {:?})",
            DURATION_SCRIPT_PATH,
            chmod_status.code()
        ));
    }

    // Execute the duration script.
    // Argument: <ogg_file_path>
    let output = Command::new(DURATION_SCRIPT_PATH)
        .arg(ogg_path.to_str().unwrap()) // Path to string conversion should be safe.
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to execute duration script {}: {}",
                DURATION_SCRIPT_PATH,
                e
            )
        })?;

    // Check if the script executed successfully.
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "{} script failed with status: {}\nStdout:\n{}\nStderr:\n{}",
            DURATION_SCRIPT_PATH,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse the duration string (expected to be a float) from script's stdout.
    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    duration_str
        .parse::<f32>()
        .map(|f| f.round() as u32) // Round to the nearest whole second.
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse duration '{}' (from {:?}) as float: {}",
                duration_str,
                ogg_path,
                e
            )
        })
}

/// Deletes unused OGG files from the `generated_assets_dir`.
/// An OGG file is considered unused if it exists in the directory but its relative path
/// is not present in the `active_ogg_paths` set (which is populated from the manifest).
/// This function will also delete empty subdirectories after removing OGG files.
///
/// # Arguments
/// * `generated_assets_dir_str` - The directory to scan for OGG files.
/// * `active_ogg_paths` - A set of OGG file paths (relative to `generated_assets_dir`,
///   including subdirectories like `base1/file.ogg`) that are currently in use.
///
/// # Returns
/// `Ok(())` on success, or an `anyhow::Error` if directory traversal or file deletion fails.
pub fn cleanup_unused_files(
    generated_assets_dir_str: &str,
    active_ogg_paths: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    println!(
        "Checking for unused OGG files in {} to delete...",
        generated_assets_dir_str
    );
    let root_dir = Path::new(generated_assets_dir_str);
    if !root_dir.exists() {
        println!(
            "Generated assets directory {} does not exist, nothing to clean up.",
            generated_assets_dir_str
        );
        return Ok(());
    }

    let mut deleted_ogg_count = 0;
    let mut potentially_empty_dirs: HashSet<PathBuf> = HashSet::new();

    // Iterate recursively to find all OGG files
    for entry in WalkDir::new(root_dir)
        .min_depth(1) // Don't include the root_dir itself in this primary scan for files.
        .into_iter()
        .filter_map(|e| e.ok())
    // Filter out read errors.
    {
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("ogg")) {
            // Get the path relative to generated_assets_dir_str
            let relative_path = path
                .strip_prefix(root_dir)?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Path is not valid UTF-8: {:?}", path))?
                .replace('\\', "/"); // Ensure consistent path separators

            if !active_ogg_paths.contains(&relative_path) {
                println!("Deleting unused OGG file: {:?}", path);
                fs::remove_file(path).map_err(|e| {
                    anyhow::anyhow!("Failed to delete unused file {:?}: {}", path, e)
                })?;
                deleted_ogg_count += 1;
                // Add parent directory to the set of potentially empty dirs
                if let Some(parent_dir) = path.parent() {
                    if parent_dir != root_dir {
                        // Don't add the root itself
                        potentially_empty_dirs.insert(parent_dir.to_path_buf());
                    }
                }
            }
        }
    }

    if deleted_ogg_count > 0 {
        println!(
            "Cleanup of {} unused OGG files complete.",
            deleted_ogg_count
        );
    } else {
        println!("No unused OGG files found to delete.");
    }

    // Attempt to delete now potentially empty subdirectories
    let mut deleted_dir_count = 0;
    // Sort directories by depth (descending) to delete children before parents
    let mut sorted_dirs: Vec<PathBuf> = potentially_empty_dirs.into_iter().collect();
    sorted_dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    for dir_path in sorted_dirs {
        // Check if directory is empty
        match fs::read_dir(&dir_path) {
            Ok(mut iter) => {
                if iter.next().is_none() {
                    // Directory is empty
                    println!("Deleting empty directory: {:?}", &dir_path);
                    fs::remove_dir(&dir_path).map_err(|e| {
                        anyhow::anyhow!("Failed to delete empty directory {:?}: {}", dir_path, e)
                    })?;
                    deleted_dir_count += 1;
                } else {
                    println!("Directory {:?} is not empty, not deleting.", &dir_path);
                }
            }
            Err(e) => {
                // It might have been deleted if it was a child of another deleted dir
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(anyhow::anyhow!(
                        "Failed to read directory {:?} for cleanup check: {}",
                        dir_path,
                        e
                    ));
                }
            }
        }
    }

    if deleted_dir_count > 0 {
        println!(
            "Cleanup of {} empty subdirectories complete.",
            deleted_dir_count
        );
    }

    Ok(())
}

/// Warns about unused OGG files in the `generated_assets_dir` without deleting them.
/// This is similar to `cleanup_unused_files` but only prints warnings and checks recursively.
///
/// # Arguments
/// * `generated_assets_dir_str` - The directory to scan for OGG files.
/// * `active_ogg_paths` - A set of OGG file paths (relative to `generated_assets_dir`,
///   including subdirectories) that are currently in use according to the manifest.
///
/// # Returns
/// `Ok(())` on success, or an `anyhow::Error` if directory traversal fails.
pub fn warn_unused_files(
    generated_assets_dir_str: &str,
    active_ogg_paths: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    println!(
        "Checking for unused OGG files (warnings only) in {}...",
        generated_assets_dir_str
    );
    let root_dir = Path::new(generated_assets_dir_str);
    if !root_dir.exists() {
        println!(
            "Generated assets directory {} does not exist, nothing to check.",
            generated_assets_dir_str
        );
        return Ok(());
    }

    let mut found_unused_count = 0;
    for entry in WalkDir::new(root_dir)
        .min_depth(1) // Don't include the root_dir itself.
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("ogg")) {
            let relative_path = path
                .strip_prefix(root_dir)?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Path is not valid UTF-8: {:?}", path))?
                .replace('\\', "/"); // Ensure consistent path separators

            if !active_ogg_paths.contains(&relative_path) {
                println!("Warning: Unused OGG file found: {:?}", path);
                found_unused_count += 1;
            }
        }
    }

    if found_unused_count == 0 {
        println!("No unused OGG files found.");
    } else {
        println!(
            "Found {} unused OGG file(s). Run with --delete-unused to remove them.",
            found_unused_count
        );
    }
    Ok(())
}

pub fn collect_audio_generation_tasks(
    ron_files: &[PathBuf],
    script_hash: &str,
    force_regenerate_pattern: Option<&str>,
) -> Result<Vec<AudioGenerationTask>, anyhow::Error> {
    let mut tasks = Vec::new();

    for ron_path in ron_files {
        let ron_filename_str = ron_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("RON path has no filename: {:?}", ron_path))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("RON filename is not valid UTF-8: {:?}", ron_path))?
            .to_string();

        let ron_file_sub_dir = ron_path
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("RON path has no filestem: {:?}", ron_path))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("RON filestem is not valid UTF-8: {:?}", ron_path))?
            .to_lowercase() // Ensure consistent directory naming
            .replace('-', "_"); // Replace hyphens with underscores for directory name

        let mut file = File::open(ron_path)
            .map_err(|e| anyhow::anyhow!("Failed to open RON file {:?}: {}", ron_path, e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| anyhow::anyhow!("Failed to read RON file {:?}: {}", ron_path, e))?;
        let phrase_file: WalkiePhraseFile = ron::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse RON file {:?}: {}", ron_path, e))?;

        for concept_entry in phrase_file.event_lines {
            for (line_idx, line_entry) in concept_entry.lines.into_iter().enumerate() {
                let conceptual_id = concept_entry.name.clone();

                // This will be the actual filename stem, e.g., "myconcept_01"
                let line_specific_filename_stem = format!(
                    "{}_{:02}",
                    conceptual_id.to_lowercase().replace('-', "_"),
                    line_idx + 1
                );

                // Relative path will include the subdirectory, e.g., "base1/myconcept_01.ogg"
                let ogg_path_relative_to_generated_dir =
                    format!("{}/{}.ogg", ron_file_sub_dir, line_specific_filename_stem);

                let ogg_path_absolute =
                    Path::new(GENERATED_ASSETS_DIR).join(&ogg_path_relative_to_generated_dir);

                let detailed_manifest_key =
                    format!("{}/{}/{}", ron_filename_str, conceptual_id, line_idx);

                tasks.push(AudioGenerationTask {
                    ron_filename_str: ron_filename_str.clone(),
                    conceptual_id,
                    line_idx,
                    line_entry,
                    script_hash: script_hash.to_string(),
                    force_regenerate_pattern: force_regenerate_pattern.map(String::from),
                    ron_file_sub_dir: ron_file_sub_dir.clone(), // Store the sub_dir
                    line_specific_filename_stem,                // Store the line-specific stem
                    ogg_path_relative_to_generated_dir,
                    ogg_path_absolute,
                    detailed_manifest_key,
                });
            }
        }
    }
    Ok(tasks)
}

/// Processes a single audio generation task. (Thread-safe version)
pub fn process_audio_generation_task(
    task: &AudioGenerationTask,
    manifest_mutex: &Arc<Mutex<HashMap<String, WalkieLineManifestEntry>>>,
    all_generated_ogg_paths_mutex: &Arc<Mutex<HashSet<String>>>,
) -> Result<(), anyhow::Error> {
    // 1. Calculate signature for current TTS text (used to check if audio content changed)
    let current_tts_signature =
        calculate_combined_signature(&task.line_entry.tts_text, &task.script_hash);

    // 2. Get existing manifest entry (if any)
    let existing_entry_opt: Option<WalkieLineManifestEntry> = {
        let manifest = manifest_mutex.lock().unwrap();
        manifest.get(&task.detailed_manifest_key).cloned()
    };

    // 3. Determine if audio needs regeneration
    let mut needs_audio_regeneration = match &existing_entry_opt {
        Some(existing_entry) => {
            existing_entry.combined_signature != current_tts_signature // Audio content changed
                || !task.ogg_path_absolute.exists() // Audio file missing
        }
        None => true, // No existing entry, so must generate audio
    };

    // Apply force_regenerate_pattern if provided
    if let Some(pattern) = &task.force_regenerate_pattern {
        if pattern == "all"
            || task.conceptual_id.contains(pattern)
            || (pattern.ends_with('*')
                && task
                    .conceptual_id
                    .starts_with(pattern.trim_end_matches('*')))
        {
            if !needs_audio_regeneration {
                // Only print if it's an override of existing up-to-date audio
                println!(
                    "Forcing audio regeneration for: {} (from {}) due to pattern '{}'",
                    task.detailed_manifest_key, task.ron_filename_str, pattern
                );
            }
            needs_audio_regeneration = true;
        }
    }

    // 4. Handle audio generation and determine actual audio duration
    let mut actual_duration_seconds: u32;

    if needs_audio_regeneration {
        println!(
            "Regenerating audio for: {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );
        let (_temp_wav_path, final_ogg_path) = generate_audio_for_line(
            &task.line_entry.tts_text,
            &task.ron_file_sub_dir,
            &task.line_specific_filename_stem,
        )?;
        actual_duration_seconds = get_audio_duration(&final_ogg_path)?;
    } else {
        // Audio is considered up-to-date, try to get duration from existing manifest or file
        actual_duration_seconds = existing_entry_opt.as_ref().map_or(0, |e| e.length_seconds);

        // If duration from manifest is 0 (or entry didn't exist but we decided not to regen),
        // and OGG file exists, try to read its duration directly.
        if actual_duration_seconds == 0 && task.ogg_path_absolute.exists() {
            println!(
                "Audio up-to-date for {}, but duration is 0 in manifest or manifest missing. Fetching duration from existing OGG: {}",
                task.detailed_manifest_key,
                task.ogg_path_absolute.display()
            );
            match get_audio_duration(&task.ogg_path_absolute) {
                Ok(d) => actual_duration_seconds = d,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to get audio duration for existing file {}: {}. Using duration 0.",
                        task.ogg_path_absolute.display(),
                        e
                    );
                    actual_duration_seconds = 0; // Fallback to 0 if reading fails
                }
            }
        }
        println!(
            "Skipping audio generation (already up-to-date): {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );
    }

    // 5. Prepare the potential new manifest entry with current data from RON
    let mut current_tags_vec: Vec<WalkieTag> = task.line_entry.tags.iter().cloned().collect();
    current_tags_vec.sort_by_key(|a| format!("{:?}", a));

    let potential_new_entry = WalkieLineManifestEntry {
        ron_file_source: task.ron_filename_str.clone(),
        conceptual_id: task.conceptual_id.clone(),
        line_index: task.line_idx,
        tts_text: task.line_entry.tts_text.clone(), // Current TTS text from RON
        subtitle_text: task.line_entry.subtitle_text.clone(), // Current subtitle from RON
        tags: current_tags_vec,                     // Current tags from RON
        ogg_path: task.ogg_path_relative_to_generated_dir.clone(),
        length_seconds: actual_duration_seconds, // Determined duration
        generation_script_hash: task.script_hash.clone(),
        combined_signature: current_tts_signature, // Signature of current TTS text
    };

    // 6. Determine if the manifest entry itself needs to be updated by comparing all fields
    let manifest_needs_update = match &existing_entry_opt {
        Some(existing_entry) => {
            // Compare all fields of the existing entry with the potential new one
            existing_entry.ron_file_source != potential_new_entry.ron_file_source
                || existing_entry.conceptual_id != potential_new_entry.conceptual_id
                || existing_entry.line_index != potential_new_entry.line_index
                || existing_entry.tts_text != potential_new_entry.tts_text
                || existing_entry.subtitle_text != potential_new_entry.subtitle_text
                || existing_entry.tags != potential_new_entry.tags
                || existing_entry.ogg_path != potential_new_entry.ogg_path
                || existing_entry.length_seconds != potential_new_entry.length_seconds
                || existing_entry.generation_script_hash
                    != potential_new_entry.generation_script_hash
                || existing_entry.combined_signature != potential_new_entry.combined_signature
        }
        None => true, // No existing entry, so manifest needs update
    };

    if manifest_needs_update {
        let mut manifest = manifest_mutex.lock().unwrap();
        manifest.insert(task.detailed_manifest_key.clone(), potential_new_entry);
        println!("Updated manifest for {}", task.detailed_manifest_key);
    } else {
        println!(
            "Manifest entry for {} is already up-to-date.",
            task.detailed_manifest_key
        );
    }

    // Always track the OGG path as active for the current run
    {
        let mut all_ogg_paths = all_generated_ogg_paths_mutex.lock().unwrap();
        all_ogg_paths.insert(task.ogg_path_relative_to_generated_dir.clone());
    }
    Ok(())
}

/// Processes a single audio generation task. (Single-threaded version)
pub fn process_audio_generation_task_single_thread(
    task: &AudioGenerationTask,
    manifest: &mut HashMap<String, WalkieLineManifestEntry>,
    all_generated_ogg_paths_from_manifest: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    // 1. Calculate signature for current TTS text
    let current_tts_signature =
        calculate_combined_signature(&task.line_entry.tts_text, &task.script_hash);

    // 2. Get existing manifest entry (if any)
    let existing_entry_opt = manifest.get(&task.detailed_manifest_key).cloned();

    // 3. Determine if audio needs regeneration
    let mut needs_audio_regeneration = match &existing_entry_opt {
        Some(existing_entry) => {
            existing_entry.combined_signature != current_tts_signature
                || !task.ogg_path_absolute.exists()
        }
        None => true,
    };

    if let Some(pattern) = &task.force_regenerate_pattern {
        if pattern == "all"
            || task.conceptual_id.contains(pattern)
            || (pattern.ends_with('*')
                && task
                    .conceptual_id
                    .starts_with(pattern.trim_end_matches('*')))
        {
            if !needs_audio_regeneration {
                println!(
                    "Forcing audio regeneration for: {} (from {}) due to pattern '{}'",
                    task.detailed_manifest_key, task.ron_filename_str, pattern
                );
            }
            needs_audio_regeneration = true;
        }
    }

    // 4. Handle audio generation and determine duration
    let mut actual_duration_seconds: u32;

    if needs_audio_regeneration {
        println!(
            "Regenerating audio for: {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );
        let (_temp_wav_path, final_ogg_path) = generate_audio_for_line(
            &task.line_entry.tts_text,
            &task.ron_file_sub_dir,
            &task.line_specific_filename_stem,
        )?;
        actual_duration_seconds = get_audio_duration(&final_ogg_path)?;
    } else {
        actual_duration_seconds = existing_entry_opt.as_ref().map_or(0, |e| e.length_seconds);
        if actual_duration_seconds == 0 && task.ogg_path_absolute.exists() {
            println!(
                "Audio up-to-date for {}, but duration is 0 in manifest or manifest missing. Fetching duration from existing OGG: {}",
                task.detailed_manifest_key,
                task.ogg_path_absolute.display()
            );
            match get_audio_duration(&task.ogg_path_absolute) {
                Ok(d) => actual_duration_seconds = d,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to get audio duration for existing file {}: {}. Using duration 0.",
                        task.ogg_path_absolute.display(),
                        e
                    );
                    actual_duration_seconds = 0;
                }
            }
        }
        println!(
            "Skipping audio generation (already up-to-date): {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );
    }

    // 5. Prepare the potential new manifest entry
    let mut current_tags_vec: Vec<WalkieTag> = task.line_entry.tags.iter().cloned().collect();
    current_tags_vec.sort_by_key(|a| format!("{:?}", a));

    let potential_new_entry = WalkieLineManifestEntry {
        ron_file_source: task.ron_filename_str.clone(),
        conceptual_id: task.conceptual_id.clone(),
        line_index: task.line_idx,
        tts_text: task.line_entry.tts_text.clone(),
        subtitle_text: task.line_entry.subtitle_text.clone(),
        tags: current_tags_vec,
        ogg_path: task.ogg_path_relative_to_generated_dir.clone(),
        length_seconds: actual_duration_seconds,
        generation_script_hash: task.script_hash.clone(),
        combined_signature: current_tts_signature,
    };

    // 6. Determine if manifest needs to be updated
    let manifest_needs_update = match &existing_entry_opt {
        Some(existing_entry) => {
            existing_entry.ron_file_source != potential_new_entry.ron_file_source
                || existing_entry.conceptual_id != potential_new_entry.conceptual_id
                || existing_entry.line_index != potential_new_entry.line_index
                || existing_entry.tts_text != potential_new_entry.tts_text
                || existing_entry.subtitle_text != potential_new_entry.subtitle_text
                || existing_entry.tags != potential_new_entry.tags
                || existing_entry.ogg_path != potential_new_entry.ogg_path
                || existing_entry.length_seconds != potential_new_entry.length_seconds
                || existing_entry.generation_script_hash
                    != potential_new_entry.generation_script_hash
                || existing_entry.combined_signature != potential_new_entry.combined_signature
        }
        None => true,
    };

    if manifest_needs_update {
        manifest.insert(task.detailed_manifest_key.clone(), potential_new_entry);
        println!("Updated manifest for {}", task.detailed_manifest_key);
    } else {
        println!(
            "Manifest entry for {} is already up-to-date.",
            task.detailed_manifest_key
        );
    }

    all_generated_ogg_paths_from_manifest.insert(task.ogg_path_relative_to_generated_dir.clone());
    Ok(())
}
