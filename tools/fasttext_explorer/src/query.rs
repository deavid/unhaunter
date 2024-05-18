//! Query Embeddings Module
//!
//! This module provides functionality for loading pre-computed FastText word embeddings and interactively querying for similar phrases.
//!
//! ## Usage
//!
//! To load embeddings from the "player" phrasebook and start the interactive query loop, run the following command from the Unhaunter project root:
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
//! You can also specify subfolders within the phrasebook directories.  For example, to load embeddings from the "player/custom" folder, use:
//!
//! ```bash
//! cargo run -p fasttext_explorer -- query-embeddings --phrasebook-type player/custom
//! ```
//!
//! ## Interactive Query Loop
//!
//! Once the embeddings are loaded, the tool will prompt you to enter a phrase.  It will then generate an embedding for the input phrase, calculate the squared Euclidean distance to all the stored embeddings, and display the top 3 closest matches.
//!
//! You can exit the query loop by pressing Ctrl-C.

use crate::{PhraseEmbedding, ASSETS_DIR, PHRASEBOOKS_DIR, VECTORS_DIR};
use fasttext::FastText;
use rand::Rng;
use serde_yaml::from_reader;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};
use walkdir::WalkDir;

// Data structures for Ghost, GhostMood, ResponseTemplate, etc.
#[derive(Debug, serde::Deserialize)]
pub struct Ghost {
    pub name: String,
    pub ghost_type: String,
    pub mood: GhostMood,
    // ... (Other metadata)
}

#[derive(Debug, serde::Deserialize)]
pub struct GhostMood(pub HashMap<String, f32>);

#[derive(Debug, serde::Deserialize)]
pub struct ResponseTemplate {
    pub trigger: ResponseTrigger,
    pub response: Response,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResponseTrigger {
    pub ghost_type: Option<String>,
    pub mood: HashMap<String, f32>,
    pub tags: Vec<String>,
    pub distance: Distance,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub enum Distance {
    #[serde(rename = "close")]
    Close,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "far")]
    Far,
    #[serde(rename = "any")]
    Any,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub enum Response {
    Text(String),
    Action(String),
    Event(String),
    Silence,
    Sound(String),
}

fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
    let magnitude1: f32 = (v1.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude2: f32 = (v2.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    dot_product / (magnitude1 * magnitude2)
}

pub fn load_ghost_metadata(project_root: &Path, filename: &str) -> Ghost {
    let filepath = project_root.join(ASSETS_DIR).join(filename);
    eprintln!("Opening Ghost Metadata at: {:?}", filepath);
    let file = File::open(filepath).unwrap();
    let ghost: Ghost = from_reader(file).unwrap();
    ghost
}

pub fn load_response_templates(project_root: &Path) -> Vec<ResponseTemplate> {
    let filepath = project_root.join(ASSETS_DIR).join("ghost_responses.yaml");
    let file = File::open(filepath).unwrap();
    let templates: Vec<ResponseTemplate> = from_reader(file).unwrap();
    templates
}

fn match_response_template<'a>(
    phrase: &'a PhraseEmbedding,
    ghost: &'a Ghost,
    distance: u32,
    templates: &'a [ResponseTemplate],
) -> Vec<&'a ResponseTemplate> {
    dbg!(templates.last());
    dbg!(&phrase.phrase, phrase.repetition_count, &phrase.tags);
    // Added lifetime specifier
    templates
        .iter()
        .filter(|template| {
            // Check ghost type
            if let Some(ghost_type) = &template.trigger.ghost_type {
                if ghost_type != "Any" && ghost_type != &ghost.ghost_type {
                    eprintln!("Ghost type: {ghost_type} != {}", ghost.ghost_type);
                    return false;
                }
            }

            // Check mood
            for (emotion, intensity) in &template.trigger.mood {
                if let Some(ghost_intensity) = ghost.mood.0.get(emotion) {
                    if *ghost_intensity < *intensity {
                        eprintln!("Intensity insufficient for {emotion:?}: ghost: {ghost_intensity} < {intensity}");
                        return false;
                    }
                } else {
                    eprintln!("Emotion/mood {emotion:?} not found in ghost: {:?}", ghost.mood);
                    return false;
                }
            }

            // Check tags
            for tag in &template.trigger.tags {
                if !phrase.tags.contains(tag) {
                    eprintln!("Phrase is missing tag {tag:?}");
                    return false;
                }
            }

            // Check distance
            match template.trigger.distance {
                Distance::Close => {
                    if distance > 5 {
                        eprintln!("Distance {distance:?} not in range for {:?}", template.trigger.distance);
                        return false;
                    }
                }
                Distance::Medium => {
                    if distance <= 5 || distance > 20 {
                        eprintln!("Distance {distance:?} not in range for {:?}", template.trigger.distance);
                        return false;
                    }
                }
                Distance::Far => {
                    if distance <= 20 {
                        eprintln!("Distance {distance:?} not in range for {:?}", template.trigger.distance);
                        return false;
                    }
                }
                Distance::Any => {}
            }

            true
        })
        .collect()
}

fn select_response_template<'a>(
    matching_templates: &'a [&'a ResponseTemplate], // Added lifetime specifier
    _ghost: &Ghost,
    phrase: &'a PhraseEmbedding, // Added lifetime specifier
) -> &'a ResponseTemplate {
    // Added lifetime specifier
    // If there's only one matching template, return it
    if matching_templates.len() == 1 {
        return matching_templates[0];
    }
    if matching_templates.is_empty() {
        panic!("Called to `select_response_template` with an empty array - that is not supported")
    }

    // If the phrase has been repeated, try to choose a different template
    if phrase.repetition_count > 1 {
        let mut rng = rand::thread_rng();
        let mut index = rng.gen_range(0..matching_templates.len());
        while matching_templates[index].response == Response::Text(phrase.phrase.clone()) {
            index = rng.gen_range(0..matching_templates.len());
        }
        return matching_templates[index];
    }

    // Otherwise, choose a random template
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..matching_templates.len());
    matching_templates[index]
}

fn generate_response(
    template: &ResponseTemplate,
    ghost: &Ghost,
    // We're currently not using the phrase argument in this function, but we
    // will need it later when we implement more sophisticated response selection
    //  logic that takes into account the phrase's content or tags.
    _phrase: &PhraseEmbedding,
) -> String {
    let mut response = match &template.response {
        Response::Text(text) => text.clone(),
        Response::Action(action) => format!("({} performs the action: {})", ghost.name, action),
        Response::Event(event) => format!("(A {} event occurs.)", event),
        Response::Silence => "... (Silence)".to_string(),
        Response::Sound(sound) => format!("(A {} sound is heard.)", sound),
    };

    // Placeholder replacement
    response = response.replace("[name]", &ghost.name);

    // Profession (random selection)
    if response.contains("[profession]") {
        let professions = ["doctor", "teacher", "artist", "musician", "writer"];
        let mut rng = rand::thread_rng();
        let profession = professions[rng.gen_range(0..professions.len())];
        response = response.replace("[profession]", profession);
    }

    // Loved one (random selection)
    if response.contains("[loved one]") {
        let loved_ones = ["mother", "father", "brother", "sister", "friend"];
        let mut rng = rand::thread_rng();
        let loved_one = loved_ones[rng.gen_range(0..loved_ones.len())];
        response = response.replace("[loved one]", loved_one);
    }

    // ... (Add replacements for other placeholders: location, player name, action, object name, room name, time of day)

    response
}

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
        if path.is_dir() {
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

    // Load response templates
    let response_templates = load_response_templates(project_root);

    // Interactive query loop
    loop {
        // Prompt for ghost metadata file and distance
        // print!("Enter a ghost metadata file: ");
        // std::io::stdout().flush().unwrap();
        // let mut ghost_metadata_file = String::new();
        // std::io::stdin()
        //     .read_line(&mut ghost_metadata_file)
        //     .unwrap();
        // let ghost_metadata_file = ghost_metadata_file.trim();
        let ghost_metadata_file = "sample_ghosts/shade.yaml";

        print!("Enter the distance from the ghost in tiles (1, 5, 10, 20, 50): ");
        std::io::stdout().flush().unwrap();
        let mut distance_input = String::new();
        std::io::stdin().read_line(&mut distance_input).unwrap();
        let distance: u32 = distance_input.trim().parse().unwrap();

        // Load ghost metadata
        let mut ghost = load_ghost_metadata(project_root, ghost_metadata_file); // Make ghost mutable

        loop {
            print!("Enter a phrase: ");
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.is_empty() {
                break; // Exit the inner loop if the input is empty
            }

            // Generate embedding for the input phrase
            let input_embedding = model.get_sentence_vector(input).unwrap();

            // Find the closest phrase in the phrasebook
            let mut closest_phrase: Option<&mut PhraseEmbedding> = None; // Store a &mut reference
            let mut closest_distance = f32::MAX;
            for (_filename, ref mut pe) in &mut embeddings {
                let distance = cosine_similarity(&input_embedding, &pe.embedding);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_phrase = Some(pe);
                }
            }

            if let Some(phrase) = closest_phrase {
                // Increment repetition count
                phrase.repetition_count += 1;

                // Match the phrase to response templates
                let matching_templates =
                    match_response_template(phrase, &ghost, distance, &response_templates);

                // Select a response template
                let template = select_response_template(&matching_templates, &ghost, phrase);

                // Generate the response
                let response = generate_response(template, &ghost, phrase);

                // Print the response
                println!("Response: {}", response);

                // Update ghost mood based on phrase tags
                for tag in &phrase.tags {
                    match tag.as_str() {
                        "Angry" | "Provocation" => ghost.mood.0.insert(
                            "anger".to_string(),
                            ghost.mood.0.get("anger").unwrap_or(&0.0) + 0.1,
                        ),
                        "Fear" | "Challenge" => ghost.mood.0.insert(
                            "fear".to_string(),
                            ghost.mood.0.get("fear").unwrap_or(&0.0) + 0.1,
                        ),
                        "Sadness" | "Reassurance" => ghost.mood.0.insert(
                            "sadness".to_string(),
                            ghost.mood.0.get("sadness").unwrap_or(&0.0) + 0.1,
                        ),
                        "Curiosity" | "Question" => ghost.mood.0.insert(
                            "curiosity".to_string(),
                            ghost.mood.0.get("curiosity").unwrap_or(&0.0) + 0.1,
                        ),
                        "Playfulness" | "Banter" => ghost.mood.0.insert(
                            "playfulness".to_string(),
                            ghost.mood.0.get("playfulness").unwrap_or(&0.0) + 0.1,
                        ),
                        _ => None,
                    };
                }

                // Normalize mood values
                let total_mood: f32 = ghost.mood.0.values().sum();
                for mood_value in ghost.mood.0.values_mut() {
                    *mood_value /= total_mood;
                }
            } else {
                println!("No matching phrase found in the phrasebook.");
            }
        }
    }
}

pub fn simulate_response(
    project_root: &Path,
    ghost_metadata_file: String,
    distance: u32,
    model: &FastText,
) {
    // Load player phrasebook embeddings
    let mut embeddings = load_embeddings(project_root, "player".to_string());

    // Load response templates
    let response_templates = load_response_templates(project_root);

    // Load ghost metadata
    let mut ghost = load_ghost_metadata(project_root, &ghost_metadata_file);

    loop {
        print!("Enter a phrase: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            break; // Exit the loop if the input is empty
        }

        // Generate embedding for the input phrase
        let input_embedding = model.get_sentence_vector(input).unwrap();

        // Find the closest phrase in the phrasebook
        let mut closest_phrase: Option<&mut PhraseEmbedding> = None; // Store a &mut reference
        let mut closest_distance = f32::MAX;
        for (_filename, ref mut pe) in &mut embeddings {
            let distance = cosine_similarity(&input_embedding, &pe.embedding);
            if distance < closest_distance {
                closest_distance = distance;
                closest_phrase = Some(pe);
            }
        }

        if let Some(phrase) = closest_phrase {
            // Increment repetition count
            phrase.repetition_count += 1;

            // Match the phrase to response templates
            let matching_templates =
                match_response_template(phrase, &ghost, distance, &response_templates);

            // Select a response template
            let template = select_response_template(&matching_templates, &ghost, phrase);

            // Generate the response
            let response = generate_response(template, &ghost, phrase);

            // Print the response
            println!("Response: {}", response);

            // Update ghost mood based on phrase tags
            for tag in &phrase.tags {
                match tag.as_str() {
                    "Angry" | "Provocation" => ghost.mood.0.insert(
                        "anger".to_string(),
                        ghost.mood.0.get("anger").unwrap_or(&0.0) + 0.1,
                    ),
                    "Fear" | "Challenge" => ghost.mood.0.insert(
                        "fear".to_string(),
                        ghost.mood.0.get("fear").unwrap_or(&0.0) + 0.1,
                    ),
                    "Sadness" | "Reassurance" => ghost.mood.0.insert(
                        "sadness".to_string(),
                        ghost.mood.0.get("sadness").unwrap_or(&0.0) + 0.1,
                    ),
                    "Curiosity" | "Question" => ghost.mood.0.insert(
                        "curiosity".to_string(),
                        ghost.mood.0.get("curiosity").unwrap_or(&0.0) + 0.1,
                    ),
                    "Playfulness" | "Banter" => ghost.mood.0.insert(
                        "playfulness".to_string(),
                        ghost.mood.0.get("playfulness").unwrap_or(&0.0) + 0.1,
                    ),
                    _ => None,
                };
            }

            // Normalize mood values
            let total_mood: f32 = ghost.mood.0.values().sum();
            for mood_value in ghost.mood.0.values_mut() {
                *mood_value /= total_mood;
            }
        } else {
            println!("No matching phrase found in the phrasebook.");
        }
    }
}

pub fn load_embeddings(
    project_root: &Path,
    phrasebook_type: String,
) -> Vec<(String, PhraseEmbedding)> {
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
        if path.is_dir() {
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

    embeddings
}
