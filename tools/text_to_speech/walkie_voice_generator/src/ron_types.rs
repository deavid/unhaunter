// Module for RON file data structures

//! RON data structures for walkie phrase files.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use unwalkie_types::WalkieTag;

/// Represents a single voice line entry within a `WalkieEventConceptEntry`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalkieLineEntry {
    /// The text to be synthesized by the TTS engine.
    pub tts_text: String,
    /// The subtitle text to be displayed in-game for this voice line.
    pub subtitle_text: String,
    /// A set of tags associated with this voice line, used for categorization or game logic.
    pub tags: HashSet<WalkieTag>,
}

/// Represents a "concept" or event that can trigger multiple voice lines.
#[derive(Serialize, Deserialize, Debug)]
pub struct WalkieEventConceptEntry {
    /// The name of the concept, in PascalCase. This will be used to generate
    /// a corresponding Rust enum variant.
    pub name: String,
    /// A list of `WalkieLineEntry` instances associated with this concept.
    pub lines: Vec<WalkieLineEntry>,
}

/// The root structure of a walkie phrase RON file.
/// Each file can contain multiple event concepts.
#[derive(Serialize, Deserialize, Debug)]
pub struct WalkiePhraseFile {
    /// A list of `WalkieEventConceptEntry` instances defined in this file.
    pub event_lines: Vec<WalkieEventConceptEntry>,
}
