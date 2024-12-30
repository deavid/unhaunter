// tools/ghost_radio/src/ghost_ai.rs
use crate::data::{EmotionalSignature, GhostResponse, PlayerPhrase};
use std::collections::HashMap;

pub fn score_responses(
    player_phrase: &PlayerPhrase,
    ghost_responses: &HashMap<String, GhostResponse>,
    ghost_mood: &EmotionalSignature,
) -> HashMap<String, f32> {
    let mut scores = HashMap::new();
    for (key, response) in ghost_responses {
        let speech_act_score = if response.for_speech_acts.contains(&player_phrase.speech_act) {
            1.0
        } else {
            0.0
        };
        let semantic_tags_score =
            cosine_similarity(&player_phrase.semantic_tags, &response.for_semantic_tags);
        let emotional_signature_score = cosine_similarity_f32(
            &emotional_signature_to_vec(ghost_mood),
            &emotional_signature_to_vec(&response.emotional_signature.emotional_signature_filter),
        );
        let seen: f32 = (response.seen_count as f32 + 2.0).powf(2.5);
        let final_score = (speech_act_score + 0.1)
            * (semantic_tags_score.clamp(0.0, 1.0) + 0.3)
            * (emotional_signature_score.clamp(0.0, 1.0) + 0.005)
            / seen;
        scores.insert(key.clone(), final_score.powf(2.1) * 4.0);
    }
    scores
}

fn emotional_signature_to_vec(es: &EmotionalSignature) -> Vec<f32> {
    vec![es.curiosity, es.fear, es.anger, es.sadness, es.joy]
}

pub fn cosine_similarity(v1: &[String], v2: &[String]) -> f32 {
    // Create sets of the unique elements in each vector
    let mut set1: std::collections::HashSet<_> = v1.iter().cloned().collect();
    let mut set2: std::collections::HashSet<_> = v2.iter().cloned().collect();
    for v in v1 {
        if v.contains(':') {
            for sv in v.split(':') {
                set1.insert(sv.to_string());
            }
        }
    }
    for v in v2 {
        if v.contains(':') {
            for sv in v.split(':') {
                set2.insert(sv.to_string());
            }
        }
    }

    // Calculate the intersection of the two sets
    let intersection: std::collections::HashSet<_> = set1.intersection(&set2).collect();

    // Calculate the magnitudes of the vectors
    let mag1 = (set1.len() as f32).sqrt();
    let mag2 = (set2.len() as f32).sqrt();

    // Calculate the cosine similarity
    (intersection.len() as f32) / (mag1 * mag2)
}

fn cosine_similarity_f32(v1: &[f32], v2: &[f32]) -> f32 {
    let dot_product = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum::<f32>();
    let mag1 = v1.iter().map(|a| a.powi(2)).sum::<f32>().sqrt();
    let mag2 = v2.iter().map(|a| a.powi(2)).sum::<f32>().sqrt();
    dot_product / (mag1 * mag2)
}
