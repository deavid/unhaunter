// tools/ghost_radio/src/data.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerPhrase {
    pub phrase: String,
    pub speech_act: String,
    pub semantic_tags: Vec<String>,
    pub emotional_signature: EmotionalSignature,
    // pub contextual_tags: Vec`<String>`, // To be added at runtime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhostResponse {
    pub phrase: String,
    pub speech_act: String,
    pub emotional_signature: GhostEmotionalSignature,
    pub response_type: String,
    pub for_speech_acts: Vec<String>,
    pub for_semantic_tags: Vec<String>,
    #[serde(default = "default_usize")]
    pub seen_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GhostMetadata {
    pub name: String,
    pub ghost_type: String,
    pub mood: EmotionalSignature,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionalSignature {
    #[serde(default = "default_f32")]
    pub curiosity: f32,
    #[serde(default = "default_f32")]
    pub fear: f32,
    #[serde(default = "default_f32")]
    pub anger: f32,
    #[serde(default = "default_f32")]
    pub sadness: f32,
    #[serde(default = "default_f32")]
    pub joy: f32,
}

fn default_f32() -> f32 {
    0.0
}

fn default_usize() -> usize {
    0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GhostEmotionalSignature {
    pub emotional_signature_filter: EmotionalSignature,
    pub emotional_signature_delta: EmotionalSignature,
}
