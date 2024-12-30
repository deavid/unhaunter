// tools/ghost_radio/src/main.rs
use data::{GhostResponse, PlayerPhrase};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde_yaml::from_reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

mod console_ui;
mod data;
mod ghost_ai;

fn main() {
    let player_phrases = load_player_phrases("assets/phrasebooks/player.yaml");
    let mut ghost_responses = load_ghost_responses("assets/phrasebooks/ghost.yaml");
    let ghosts = ["poltergeist", "shade"];
    console_ui::display_ghost_options(&ghosts);
    let ghost_choice = get_user_choice();
    let selected_ghost = ghosts[ghost_choice - 1].to_owned();
    let ghost_metadata =
        load_ghost_metadata(&format!("assets/sample_ghosts/{selected_ghost}.yaml"));
    let mut ghost_mood = ghost_metadata.mood.clone();
    let original_ghost_mood = ghost_mood.clone();
    let phrases = player_phrases.keys().cloned().collect::<Vec<_>>();
    println!("Available Phrases:");
    for (i, key) in phrases.iter().enumerate() {
        println!("{}. {}", i + 1, key);
    }
    loop {
        println!(
            "Ghost Mood: anger {:.2}, curiosity {:.2}, fear {:.2}, joy {:.2}, sadness {:.2}",
            ghost_mood.anger,
            ghost_mood.curiosity,
            ghost_mood.fear,
            ghost_mood.joy,
            ghost_mood.sadness
        );
        let player_phrase = console_ui::get_player_phrase(&phrases);
        println!();
        println!("Player says: {}", player_phrase);
        let scores = ghost_ai::score_responses(
            &player_phrases[&player_phrase],
            &ghost_responses,
            &ghost_mood,
        );
        let mut sc = scores
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>();
        sc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // for (resp, score) in sc.into_iter().take(25) { eprintln!("-> {score:.3} ::
        // {resp}"); if score < 0.1 { break; } } Weighted random selection
        let mut rng = thread_rng();
        let weights: Vec<f32> = sc
            .iter()
            .map(|(_, s)| s.clamp(0.00000001, 9999.9))
            .collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let chosen_index = dist.sample(&mut rng);
        let chosen_response_key = sc.get(chosen_index).unwrap();
        ghost_responses
            .get_mut(&chosen_response_key.0)
            .unwrap()
            .seen_count += 1;
        let response = &ghost_responses[&chosen_response_key.0];
        console_ui::display_ghost_response(response);
        println!();

        // Update ghost mood
        let resd = &response.emotional_signature.emotional_signature_delta;
        const MOOD_F: f32 = 5.0;
        ghost_mood.curiosity += resd.curiosity * MOOD_F;
        ghost_mood.fear += resd.fear * MOOD_F;
        ghost_mood.anger += resd.anger * MOOD_F;
        ghost_mood.sadness += resd.sadness * MOOD_F;
        ghost_mood.joy += resd.joy * MOOD_F;

        // Ghost mood returns to the mean after N turns
        const RETURN_TO_MEAN_TURNS: f32 = 4.0;
        const F1: f32 = 1.0 / RETURN_TO_MEAN_TURNS;
        const F2: f32 = 1.0 - F1;
        ghost_mood.curiosity =
            original_ghost_mood.curiosity * F1 + ghost_mood.curiosity.clamp(0.0, 1.0) * F2;
        ghost_mood.fear = original_ghost_mood.fear * F1 + ghost_mood.fear.clamp(0.0, 1.0) * F2;
        ghost_mood.anger = original_ghost_mood.anger * F1 + ghost_mood.anger.clamp(0.0, 1.0) * F2;
        ghost_mood.sadness =
            original_ghost_mood.sadness * F1 + ghost_mood.sadness.clamp(0.0, 1.0) * F2;
        ghost_mood.joy = original_ghost_mood.joy * F1 + ghost_mood.joy.clamp(0.0, 1.0) * F2;
    }
}

fn load_player_phrases(filepath: &str) -> HashMap<String, PlayerPhrase> {
    let file = File::open(filepath).unwrap();
    let reader = BufReader::new(file);
    let phrases: Vec<data::PlayerPhrase> = from_reader(reader).unwrap();
    phrases.into_iter().map(|p| (p.phrase.clone(), p)).collect()
}

fn load_ghost_responses(filepath: &str) -> HashMap<String, GhostResponse> {
    let file = File::open(filepath).unwrap();
    let reader = BufReader::new(file);
    let responses: Vec<data::GhostResponse> = from_reader(reader).unwrap();
    responses
        .into_iter()
        .map(|r| (r.phrase.clone(), r))
        .collect()
}

fn load_ghost_metadata(filepath: &str) -> data::GhostMetadata {
    let file = File::open(filepath)
        .unwrap_or_else(|e| panic!("load_ghost_metadata({filepath:?}) => {e:?}"));
    let reader = BufReader::new(file);
    let metadata: data::GhostMetadata = from_reader(reader).unwrap();
    metadata
}

fn get_user_choice() -> usize {
    loop {
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            if let Ok(choice) = input.trim().parse::<usize>() {
                return choice;
            } else {
                println!("Invalid input. Please enter a number.");
            }
        } else {
            println!("Error reading input.");
        }
    }
}
