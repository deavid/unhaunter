// Module for processing RON files and generating audio

//! Functions for processing RON files, generating audio, and managing generated files.

use crate::constants::{
    DURATION_SCRIPT_PATH, GENERATE_SCRIPT_PATH, GENERATED_ASSETS_DIR, TEMP_AUDIO_DIR,
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
    pub ogg_base_filename: String,
    pub ogg_path_relative_to_generated_dir: String,
    pub ogg_path_absolute: PathBuf,
    pub detailed_manifest_key: String,
}

/// Generates an audio file for a given TTS text and base filename.
/// It calls the `GENERATE_SCRIPT_PATH` shell script which handles TTS and ffmpeg processing.
///
/// # Arguments
/// * `tts_text` - The text to synthesize.
/// * `ogg_base_filename` - The base name for the output files (e.g., "concept_line_01"),
///   `.wav` and `.ogg` extensions will be appended.
///
/// # Returns
/// A `Result` containing a tuple of `(PathBuf, PathBuf)` for the temporary WAV path
/// and final OGG path respectively, or an `anyhow::Error` if script execution fails.
pub fn generate_audio_for_line(
    tts_text: &str,
    ogg_base_filename: &str,
) -> Result<(PathBuf, PathBuf), anyhow::Error> {
    // Define paths for temporary WAV and final OGG files.
    let temp_wav_filename = format!("{}.wav", ogg_base_filename);
    let temp_wav_path = Path::new(TEMP_AUDIO_DIR).join(temp_wav_filename);
    let final_ogg_filename = format!("{}.ogg", ogg_base_filename);
    let final_ogg_path = Path::new(GENERATED_ASSETS_DIR).join(final_ogg_filename);

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
///
/// # Arguments
/// * `generated_assets_dir_str` - The directory to scan for OGG files.
/// * `active_ogg_paths` - A set of OGG file paths (relative to `generated_assets_dir`)
///   that are currently in use according to the manifest.
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
    let dir = Path::new(generated_assets_dir_str);
    if !dir.exists() {
        println!(
            "Generated assets directory {} does not exist, nothing to clean up.",
            generated_assets_dir_str
        );
        return Ok(());
    }

    let mut deleted_count = 0;
    // Iterate over files in the generated_assets_dir (non-recursive).
    for entry in WalkDir::new(dir)
        .min_depth(1) // Don't include the directory itself.
        .max_depth(1) // Only files directly in the directory.
        .into_iter()
        .filter_map(|e| e.ok())
    // Filter out read errors.
    {
        let path = entry.path();
        // Check if it's an OGG file.
        if path.is_file() && path.extension() == Some(OsStr::new("ogg")) {
            // Get the filename part to compare with active_ogg_paths.
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Path has no filename: {:?}", path))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Filename is not valid UTF-8: {:?}", path))?
                .to_string();

            if !active_ogg_paths.contains(&file_name) {
                println!("Deleting unused OGG file: {:?}", path);
                fs::remove_file(path).map_err(|e| {
                    anyhow::anyhow!("Failed to delete unused file {:?}: {}", path, e)
                })?;
                deleted_count += 1;
            }
        }
    }

    if deleted_count > 0 {
        println!("Cleanup of {} unused OGG files complete.", deleted_count);
    } else {
        println!("No unused OGG files found to delete.");
    }
    Ok(())
}

/// Warns about unused OGG files in the `generated_assets_dir` without deleting them.
/// This is similar to `cleanup_unused_files` but only prints warnings.
///
/// # Arguments
/// * `generated_assets_dir_str` - The directory to scan for OGG files.
/// * `active_ogg_paths` - A set of OGG file paths (relative to `generated_assets_dir`)
///   that are currently in use according to the manifest.
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
    let dir = Path::new(generated_assets_dir_str);
    if !dir.exists() {
        println!(
            "Generated assets directory {} does not exist, nothing to check.",
            generated_assets_dir_str
        );
        return Ok(());
    }

    let mut found_unused_count = 0;
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("ogg")) {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Path has no filename: {:?}", path))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Filename is not valid UTF-8: {:?}", path))?
                .to_string();

            if !active_ogg_paths.contains(&file_name) {
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

                let ogg_base_filename = format!(
                    "{}_{}_{:02}",
                    ron_path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .replace('-', "_"),
                    conceptual_id.to_lowercase().replace('-', "_"),
                    line_idx + 1
                );
                let ogg_filename = format!("{}.ogg", ogg_base_filename);
                let ogg_path_relative_to_generated_dir = ogg_filename.clone();
                let ogg_path_absolute = Path::new(GENERATED_ASSETS_DIR).join(&ogg_filename);

                let detailed_manifest_key =
                    format!("{}/{}/{}", ron_filename_str, conceptual_id, line_idx);

                tasks.push(AudioGenerationTask {
                    ron_filename_str: ron_filename_str.clone(),
                    conceptual_id,
                    line_idx,
                    line_entry,
                    script_hash: script_hash.to_string(),
                    force_regenerate_pattern: force_regenerate_pattern.map(String::from),
                    ogg_base_filename,
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
    let combined_signature =
        calculate_combined_signature(&task.line_entry.tts_text, &task.script_hash);

    let mut needs_regeneration = true;
    // Check against existing manifest entry
    {
        let manifest = manifest_mutex.lock().unwrap();
        if let Some(existing_entry) = manifest.get(&task.detailed_manifest_key) {
            if existing_entry.combined_signature == combined_signature
                && task.ogg_path_absolute.exists()
            {
                needs_regeneration = false;
            }
        }
    }

    // Check for forced regeneration
    if let Some(pattern) = &task.force_regenerate_pattern {
        if pattern == "all"
            || task.conceptual_id.contains(pattern)
            || (pattern.ends_with('*')
                && task
                    .conceptual_id
                    .starts_with(pattern.trim_end_matches('*')))
        {
            needs_regeneration = true;
            println!(
                "Forcing regeneration for: {} (from {}) due to pattern '{}'",
                task.detailed_manifest_key, task.ron_filename_str, pattern
            );
        }
    }

    if needs_regeneration {
        println!(
            "Regenerating audio for: {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );

        let (_temp_wav_path, final_ogg_path) =
            generate_audio_for_line(&task.line_entry.tts_text, &task.ogg_base_filename)?;
        let duration_seconds = get_audio_duration(&final_ogg_path)?;

        let mut tags_vec: Vec<WalkieTag> = task.line_entry.tags.iter().cloned().collect();
        tags_vec.sort_by_key(|a| format!("{:?}", a));

        let new_entry = WalkieLineManifestEntry {
            ron_file_source: task.ron_filename_str.clone(),
            conceptual_id: task.conceptual_id.clone(),
            line_index: task.line_idx,
            tts_text: task.line_entry.tts_text.clone(),
            subtitle_text: task.line_entry.subtitle_text.clone(),
            tags: tags_vec,
            ogg_path: task.ogg_path_relative_to_generated_dir.clone(),
            length_seconds: duration_seconds,
            generation_script_hash: task.script_hash.clone(),
            combined_signature,
        };
        // Update manifest
        {
            let mut manifest = manifest_mutex.lock().unwrap();
            manifest.insert(task.detailed_manifest_key.clone(), new_entry);
        }
        println!("Updated manifest for {}", task.detailed_manifest_key);
    } else {
        println!(
            "Skipping audio generation (already up-to-date): {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );
    }

    // Track OGG path
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
    let combined_signature =
        calculate_combined_signature(&task.line_entry.tts_text, &task.script_hash);

    let mut needs_regeneration = true;
    if let Some(existing_entry) = manifest.get(&task.detailed_manifest_key) {
        if existing_entry.combined_signature == combined_signature
            && task.ogg_path_absolute.exists()
        {
            needs_regeneration = false;
        }
    }

    if let Some(pattern) = &task.force_regenerate_pattern {
        if pattern == "all"
            || task.conceptual_id.contains(pattern)
            || (pattern.ends_with('*')
                && task
                    .conceptual_id
                    .starts_with(pattern.trim_end_matches('*')))
        {
            needs_regeneration = true;
            println!(
                "Forcing regeneration for: {} (from {}) due to pattern '{}'",
                task.detailed_manifest_key, task.ron_filename_str, pattern
            );
        }
    }

    if needs_regeneration {
        println!(
            "Regenerating audio for: {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );

        let (_temp_wav_path, final_ogg_path) =
            generate_audio_for_line(&task.line_entry.tts_text, &task.ogg_base_filename)?;
        let duration_seconds = get_audio_duration(&final_ogg_path)?;

        let mut tags_vec: Vec<WalkieTag> = task.line_entry.tags.iter().cloned().collect();
        tags_vec.sort_by_key(|a| format!("{:?}", a));

        let new_entry = WalkieLineManifestEntry {
            ron_file_source: task.ron_filename_str.clone(),
            conceptual_id: task.conceptual_id.clone(),
            line_index: task.line_idx,
            tts_text: task.line_entry.tts_text.clone(),
            subtitle_text: task.line_entry.subtitle_text.clone(),
            tags: tags_vec,
            ogg_path: task.ogg_path_relative_to_generated_dir.clone(),
            length_seconds: duration_seconds,
            generation_script_hash: task.script_hash.clone(),
            combined_signature,
        };
        manifest.insert(task.detailed_manifest_key.clone(), new_entry);
        println!("Updated manifest for {}", task.detailed_manifest_key);
    } else {
        println!(
            "Skipping audio generation (already up-to-date): {} (from {}), line {}",
            task.conceptual_id, task.ron_filename_str, task.line_idx
        );
    }
    all_generated_ogg_paths_from_manifest.insert(task.ogg_path_relative_to_generated_dir.clone());
    Ok(())
}
