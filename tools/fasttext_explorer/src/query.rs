//! Query Embeddings Module
//!
//! This module provides functionality for loading pre-computed FastText word
//! embeddings and interactively querying for similar phrases.
//!
//! ## Usage
//!
//! To load embeddings from the "player" phrasebook and start the interactive
//! query loop, run the following command from the Unhaunter project root:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- query-embeddings --phrasebook-type player
//! ```
//!
//! To load embeddings from the "ghost" phrasebook, use:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- query-embeddings --phrasebook-type ghost
//! ```
//!
//! You can also specify subfolders within the phrasebook directories.
//! For example, to load embeddings from the "player/custom" folder, use:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- query-embeddings --phrasebook-type player/custom
//! ```
//!
//! ## Interactive Query Loop
//!
//! Once the embeddings are loaded, the tool will prompt you to enter a phrase.
//! It will then generate an embedding for the input phrase, calculate the
//! squared Euclidean distance to all the stored embeddings, and display the
//! top 3 closest matches.
//!
//! You can exit the query loop by pressing Ctrl-C.

use std::{
    fs::File,
    io::{BufRead as _, BufReader, Write as _},
    path::{Path, PathBuf},
};

use fasttext::FastText;
use walkdir::WalkDir;

use crate::{PhraseEmbedding, ASSETS_DIR, JSONL_EXTENSION, PHRASEBOOKS_DIR, VECTORS_DIR};

pub fn query_embeddings(project_root: &Path, phrasebook_type: String, model: &FastText) {
    // Define the directory containing the embedding files
    let embeddings_dir = project_root
        .join(ASSETS_DIR)
        .join(PHRASEBOOKS_DIR)
        .join(VECTORS_DIR)
        .join(phrasebook_type);

    // Load embeddings into memory
    let mut embeddings: Vec<(String, PhraseEmbedding)> = Vec::new();
    for entry in WalkDir::new(embeddings_dir) {
        let entry = entry.unwrap();
        let path = entry.path();

        // Skip directories
        if path.is_dir() || path.extension().unwrap_or_default() != JSONL_EXTENSION {
            continue;
        }

        // Load the JSONL file
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let embedding: PhraseEmbedding = serde_json::from_str(&line).unwrap();
            embeddings.push((path.to_string_lossy().to_string(), embedding));
        }
    }

    // Interactive query loop
    loop {
        print!("Enter a phrase: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        // Generate embedding for the input phrase
        let input_embedding = model.get_sentence_vector(input).unwrap();

        // Calculate distances and find the top 3 closest phrases
        let mut distances: Vec<(f32, String, String)> = embeddings
            .iter()
            .map(|(filename, pe)| {
                let distance = cosine_similarity(&input_embedding, &pe.embedding);
                (distance, pe.phrase.clone(), filename.clone())
            })
            .collect();

        // Sort by ascending distance
        distances.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Print the top closest phrases
        println!("Top 8 closest phrases:");
        for (i, (distance, phrase, filename)) in distances.iter().take(8).enumerate() {
            let filename = PathBuf::from(filename)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            println!(
                "{}. Similarity: {:.4}, Phrase: '{}', File: {}",
                i + 1,
                distance,
                phrase,
                filename
            );
        }
    }
}

fn _squared_euclidean_distance(v1: &[f32], v2: &[f32]) -> f32 {
    v1.iter().zip(v2.iter()).map(|(a, b)| (a - b).powi(2)).sum()
}

fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
    let magnitude1: f32 = (v1.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude2: f32 = (v2.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    dot_product / (magnitude1 * magnitude2)
}
