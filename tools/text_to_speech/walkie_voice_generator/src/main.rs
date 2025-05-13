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

    if cli.delete_unused {
        println!("Cleaning manifest of entries from deleted RON files...");
        // ron_files is Vec<PathBuf> from scan_ron_files earlier
        let active_ron_filenames: HashSet<String> = ron_files
            .iter()
            .map(|path_buf| {
                path_buf
                    .file_name()
                    .unwrap_or_default() // Should always have a filename for valid ron_files
                    .to_str()
                    .unwrap_or_default() // Filenames should be valid UTF-8
                    .to_string()
            })
            .collect();

        let original_manifest_size = manifest_final.len();
        manifest_final.retain(|_key, manifest_entry| {
            active_ron_filenames.contains(&manifest_entry.ron_file_source)
        });
        let cleaned_manifest_size = manifest_final.len();
        if original_manifest_size > cleaned_manifest_size {
            println!(
                "Removed {} stale entries from the manifest.",
                original_manifest_size - cleaned_manifest_size
            );
        } else {
            println!("No stale entries found in the manifest.");
        }
    }

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

    fs::remove_dir_all(TEMP_AUDIO_DIR).expect("Failed to remove temporary audio directory");
    fs::create_dir_all(TEMP_AUDIO_DIR).expect("Failed to re-create temporary audio directory");

    println!("Walkie voice generation complete.");
}

// All original functions and structs have been moved to their respective modules.
// main.rs now only contains the main function, module declarations, and necessary top-level use statements.
