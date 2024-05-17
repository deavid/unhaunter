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
}

// Define a struct to represent a phrase and its embedding
#[derive(Debug, Serialize, Deserialize)]
struct PhraseEmbedding {
    phrase: String,
    embedding: Vec<f32>,
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
    }
}
