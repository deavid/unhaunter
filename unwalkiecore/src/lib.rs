pub mod events;
pub mod generated;
pub mod resources;

use unwalkie_types::VoiceLineData;

/// Define a common trait for all generated concepts
/// This allows `to_concept` to return a Box<dyn ConceptTrait>
/// and `sound_file_list` to call `get_lines()` on it.
pub trait ConceptTrait {
    fn get_lines(&self) -> Vec<VoiceLineData>;
}

pub use events::{WalkieEvent, WalkieEventPriority};
pub use resources::{WalkieEventStats, WalkiePlay, WalkieSoundState};
