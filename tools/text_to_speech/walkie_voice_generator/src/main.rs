//! # Walkie Voice Generator
//!
//! This CLI tool automates the generation of walkie-talkie style voice lines for the Unhaunter game.
//! It processes RON (Rusty Object Notation) files containing text-to-speech definitions,
//! generates OGG audio files with applied effects, manages these files via a manifest,
//! and generates corresponding Rust code for use in the game.

// Declare modules
mod cli;
mod codegen;
mod constants;
mod file_processing;
mod manifest_handling;
mod manifest_types;
mod ron_types;
mod utils;

use crate::cli::Cli;
use crate::codegen::generate_rust_code;
use crate::constants::*;
use crate::file_processing::{
    cleanup_unused_files, collect_audio_generation_tasks, process_audio_generation_task,
    process_audio_generation_task_single_thread, warn_unused_files,
};
use crate::manifest_handling::{load_manifest, save_manifest};
use crate::utils::{calculate_script_hash, generate_sample_ron, scan_ron_files};

use clap::Parser;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};

// --- Main Application Logic ---

/// Main entry point for the `walkie_voice_generator` CLI application.
fn main() {
    // Parse command-line arguments.
    let cli = Cli::parse();

    // Ensure critical output directories exist.
    fs::create_dir_all(GENERATED_ASSETS_DIR).expect("Failed to create generated assets directory");
    fs::create_dir_all(GENERATED_RUST_DIR).expect("Failed to create generated Rust directory");
    fs::create_dir_all(TEMP_AUDIO_DIR).expect("Failed to create temporary audio directory");

    // Handle `--generate-sample-ron` flag.
    if cli.generate_sample_ron {
        let sample_ron_string = generate_sample_ron();
        println!("{}", sample_ron_string);
        return;
    }

    // Load the existing manifest or create a new one if it doesn't exist.
    let manifest = load_manifest().unwrap_or_else(|err| {
        // from crate::manifest_handling
        eprintln!(
            "Warning: Could not load manifest, starting with an empty one. Error: {}",
            err
        );
        HashMap::new()
    });

    // Calculate the hash of the audio generation script.
    let script_hash =
        calculate_script_hash(GENERATE_SCRIPT_PATH).expect("Failed to hash generation script");

    // Scan the `WALKIE_PHRASES_DIR` for input RON files.
    let ron_files = scan_ron_files(WALKIE_PHRASES_DIR).expect("Failed to scan RON files");

    // Collect all audio generation tasks from all RON files.
    let audio_tasks = match collect_audio_generation_tasks(
        &ron_files,
        &script_hash,
        cli.force_regenerate.as_deref(),
    ) {
        Ok(tasks) => tasks,
        Err(e) => {
            eprintln!("Error collecting audio generation tasks: {}", e);
            return;
        }
    };

    // Create a set of all detailed_manifest_keys from the current RON definitions.
    // This will be used to prune the manifest of any entries that no longer correspond
    // to a defined voice line.
    let current_defined_keys: HashSet<String> = audio_tasks
        .iter()
        .map(|task| task.detailed_manifest_key.clone())
        .collect();

    let all_generated_ogg_paths = Arc::new(Mutex::new(HashSet::<String>::new()));
    let manifest_mutex = Arc::new(Mutex::new(manifest));

    if cli.parallel_jobs > 1 {
        println!(
            "Processing {} audio tasks in parallel with {} jobs",
            audio_tasks.len(),
            cli.parallel_jobs
        );
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.parallel_jobs)
            .build_global()
            .expect("Failed to build thread pool");

        audio_tasks.par_iter().for_each(|task| {
            // println!("Processing task in parallel: {:?}", task.detailed_manifest_key);
            let result =
                process_audio_generation_task(task, &manifest_mutex, &all_generated_ogg_paths);
            if let Err(e) = result {
                eprintln!(
                    "Error processing task {} for {}: {}",
                    task.line_idx, task.conceptual_id, e
                );
            }
        });
    } else {
        println!("Processing {} audio tasks sequentially", audio_tasks.len());
        for task in audio_tasks {
            // println!("Processing task: {:?}", task.detailed_manifest_key);
            let mut manifest_guard = manifest_mutex.lock().unwrap();
            let mut paths_guard = all_generated_ogg_paths.lock().unwrap();
            if let Err(e) = process_audio_generation_task_single_thread(
                &task, // Pass the task
                &mut manifest_guard,
                &mut paths_guard,
            ) {
                eprintln!(
                    "Error processing task {} for {}: {}",
                    task.line_idx, task.conceptual_id, e
                );
            }
        }
    }

    let mut manifest_final = Arc::try_unwrap(manifest_mutex)
        .expect("Failed to unwrap manifest Arc")
        .into_inner()
        .expect("Failed to unwrap manifest Mutex");

    let all_generated_ogg_paths_final = Arc::try_unwrap(all_generated_ogg_paths)
        .expect("Failed to unwrap paths Arc")
        .into_inner()
        .expect("Failed to unwrap paths Mutex");

    // Prune manifest to remove entries for lines/concepts no longer defined in any RON file
    // or lines/concepts removed from existing RON files. This ensures the manifest accurately
    // reflects the current state of definitions.
    println!("Synchronizing manifest with current RON definitions...");
    let original_manifest_size_before_pruning = manifest_final.len();
    manifest_final.retain(|key, _entry| current_defined_keys.contains(key));
    let new_manifest_size_after_pruning = manifest_final.len();

    if original_manifest_size_before_pruning > new_manifest_size_after_pruning {
        println!(
            "Removed {} stale entries from the manifest (lines/concepts no longer defined, or from deleted RON files).",
            original_manifest_size_before_pruning - new_manifest_size_after_pruning
        );
    } else {
        println!("Manifest is already synchronized with current RON definitions.");
    }

    // The `if cli.delete_unused` block that previously handled manifest cleaning
    // based on deleted RON files is now effectively superseded by the more comprehensive
    // pruning logic above. The `cli.delete_unused` flag will still control
    // OGG file cleanup and Rust code generation aspects.

    save_manifest(&manifest_final).expect("Failed to save manifest"); // from crate::manifest_handling

    generate_rust_code(&manifest_final, GENERATED_RUST_DIR, cli.delete_unused)
        .expect("Failed to generate Rust code"); // from crate::codegen

    if cli.delete_unused {
        cleanup_unused_files(GENERATED_ASSETS_DIR, &all_generated_ogg_paths_final)
            .expect("Failed to cleanup unused files");
    } else {
        warn_unused_files(GENERATED_ASSETS_DIR, &all_generated_ogg_paths_final)
            .expect("Failed to check for unused files");
    }

    // Clean up temporary audio files (*.wav, *.ogg) from TEMP_AUDIO_DIR non-recursively
    println!(
        "Cleaning up temporary audio files (*.wav, *.ogg) from {}.",
        TEMP_AUDIO_DIR
    );
    match fs::read_dir(TEMP_AUDIO_DIR) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext_osstr) = path.extension() {
                        if let Some(ext) = ext_osstr.to_str() {
                            if ext == "wav" || ext == "ogg" {
                                println!("Deleting temporary file: {:?}", path);
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!(
                                        "Warning: Failed to delete temporary file {:?}: {}",
                                        path, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not read temporary audio directory {} for cleanup: {}",
                TEMP_AUDIO_DIR, e
            );
        }
    }
    // The directory TEMP_AUDIO_DIR itself is no longer removed and recreated here.
    // It's created if it doesn't exist at the beginning of main().

    println!("Walkie voice generation complete.");
}

// All original functions and structs have been moved to their respective modules.
// main.rs now only contains the main function, module declarations, and necessary top-level use statements.
