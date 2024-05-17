//! Embeddings Module
//!
//! This module provides functionality for generating and storing FastText word
//! embeddings for phrases in the Unhaunter phrasebook.
//!
//! ## Usage
//!
//! To generate embeddings for the "player" phrasebook, run the following
//! command from the Unhaunter project root:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- generate-embeddings --phrasebook-type player
//! ```
//!
//! To generate embeddings for the "ghost" phrasebook, use:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- generate-embeddings --phrasebook-type ghost
//! ```
//!
//! You can also specify subfolders within the phrasebook directories. 
//! For example, to process phrases in the "player/custom" folder, use:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- generate-embeddings --phrasebook-type player/custom
//! ```
//!
//! ## Options
//!
//! * `--no-overwrite`:  Don't overwrite existing embedding files.
//! * `--process-newer`:  Only process files where the source YAML file is newer than the destination JSONL file.  (Default: true)

use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

use fasttext::FastText;
use walkdir::WalkDir;

use crate::{
    PhraseEmbedding, ASSETS_DIR, JSONL_EXTENSION, PHRASEBOOKS_DIR, VECTORS_DIR, YAML_EXTENSION,
};

pub fn process_embeddings(
    project_root: &PathBuf,
    phrasebook_type: String,
    no_overwrite: bool,
    process_newer: bool,
) {
    // Define the source and destination directories
    let source_dir = project_root
        .join(ASSETS_DIR)
        .join(PHRASEBOOKS_DIR)
        .join(phrasebook_type.clone());
    let dest_dir = project_root
        .join(ASSETS_DIR)
        .join(PHRASEBOOKS_DIR)
        .join(VECTORS_DIR)
        .join(phrasebook_type);

    // Load the FastText model
    let mut model = FastText::new();
    model
        .load_model("assets/phrasebooks/vectors/crawl-300d-2M-subword.bin")
        .unwrap();

    // Iterate through YAML files in the source directory
    for entry in WalkDir::new(source_dir) {
        let entry = entry.unwrap();
        let path = entry.path();

        // Skip directories and the index.yaml file
        if path.is_dir() || path.extension().unwrap_or_default() != YAML_EXTENSION {
            continue;
        }

        // Get the relative path for the destination file
        let relative_path = path.strip_prefix(project_root).unwrap();
        let dest_path = dest_dir.join(relative_path).with_extension(JSONL_EXTENSION);

        // Check if the file should be processed based on overwrite and newer flags
        if no_overwrite && dest_path.exists() {
            println!("Skipping file (no_overwrite): {:?}", path);
            continue;
        }
        if process_newer
            && dest_path.exists()
            && path.metadata().unwrap().modified().unwrap()
                <= dest_path.metadata().unwrap().modified().unwrap()
        {
            println!("Skipping file (not newer): {:?}", path);
            continue;
        }

        // Load the YAML file
        let file = File::open(path).unwrap();
        let phrases: Vec<String> = serde_yaml::from_reader(file).unwrap();

        // Generate embeddings and store them in a vector
        let embeddings: Vec<PhraseEmbedding> = phrases
            .iter()
            .map(|phrase| PhraseEmbedding {
                phrase: phrase.clone(),
                embedding: model.get_sentence_vector(phrase).unwrap(),
            })
            .collect();

        // Create the destination directory if it doesn't exist
        create_dir_all(dest_path.parent().unwrap()).unwrap();

        // Serialize the embeddings to a JSONL file
        let dest_file = File::create(dest_path).unwrap();
        serde_yaml::to_writer(dest_file, &embeddings).unwrap();

        println!("Processed file: {:?}", path);
    }

    println!("Embedding generation complete!");
}
