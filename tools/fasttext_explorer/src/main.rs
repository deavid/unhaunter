pub mod embeddings;
pub mod query;

use clap::{Parser, Subcommand};

use fasttext::FastText;
use serde::{Deserialize, Serialize};

use std::env;

// Constants for file paths and extensions
const ASSETS_DIR: &str = "assets";
const PHRASEBOOKS_DIR: &str = "phrasebooks";
const VECTORS_DIR: &str = "vectors";
const YAML_EXTENSION: &str = "yaml";
const JSONL_EXTENSION: &str = "jsonl";
const MODEL_PATH: &str = "assets/phrasebooks/vectors/crawl-300d-2M-subword.bin";

// Define your command-line arguments using clap
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Generate embeddings and store them in JSONL files
    GenerateEmbeddings {
        /// The phrasebook type to process (e.g., "player", "ghost")
        #[arg(short, long)]
        phrasebook_type: String,
        /// Don't overwrite existing embedding files
        #[arg(short, long)]
        no_overwrite: bool,
        /// Only process files where the source is newer than the destination
        #[arg(short, long)]
        process_newer: bool,
    },
    /// Load embeddings and query for similar phrases
    QueryEmbeddings {
        /// The phrasebook type to load (e.g., "player", "ghost")
        #[arg(short, long)]
        phrasebook_type: String,
    },
    /// Simulate ghost responses to player phrases
    SimulateResponse {
        /// Path to the ghost metadata YAML file
        #[arg(short, long)]
        ghost_metadata_file: String,
        /// Distance from the ghost in tiles (1, 5, 10, 20, 50)
        #[arg(short, long)]
        distance: u32,
    },
}

// Define a struct to represent a phrase and its embedding
#[derive(Debug, Serialize, Deserialize)]
pub struct PhraseEmbedding {
    pub phrase: String,
    pub embedding: Vec<f32>,
    pub tags: Vec<String>,
    pub repetition_count: u32,
}

fn main() {
    // Parse command-line arguments
    let args = Args::parse();

    // Get the absolute path to the project root
    let project_root = env::current_dir().unwrap();
    eprintln!("Loading model {:?}", MODEL_PATH);

    // Load the FastText model (only once)
    let mut model = FastText::new();
    model.load_model(MODEL_PATH).unwrap();
    eprintln!("Loading model -> Done");

    // Match the selected action
    match args.action {
        Action::GenerateEmbeddings {
            phrasebook_type,
            no_overwrite,
            process_newer,
        } => embeddings::process_embeddings(
            &project_root,
            phrasebook_type,
            no_overwrite,
            process_newer,
            &model,
        ),
        Action::QueryEmbeddings { phrasebook_type } => {
            query::query_embeddings(&project_root, phrasebook_type, &model)
        }
        Action::SimulateResponse {
            ghost_metadata_file,
            distance,
        } => query::simulate_response(&project_root, ghost_metadata_file, distance, &model),
    }
}
