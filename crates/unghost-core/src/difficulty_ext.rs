//! Extension trait for Difficulty to access ghost-related settings
//!
//! This trait breaks the dependency inversion: instead of undifficulty-core depending on unghost-core,
//! unghost-core now provides this extension trait to define what ghosts are available for each difficulty.

use crate::types::ghost::definitions::GhostSet;
use untypes_core::difficulty::Difficulty;

/// Extension trait providing ghost-related difficulty settings.
///
/// This trait is implemented by `Difficulty` to provide ghost set information without creating a
/// dependency from undifficulty-core to unghost-core.
pub trait DifficultyGhostExt {
    /// Returns the set of ghosts available for this difficulty.
    fn ghost_set(&self) -> GhostSet;
}

impl DifficultyGhostExt for Difficulty {
    fn ghost_set(&self) -> GhostSet {
        match self {
            Difficulty::TutorialChapter1 => GhostSet::TmpEMF,
            Difficulty::TutorialChapter2 => GhostSet::TmpEMFUVOrbs,
            Difficulty::TutorialChapter3 => GhostSet::TmpEMFUVOrbsEVPCPM,
            Difficulty::TutorialChapter4 => GhostSet::Twenty,
            _ => GhostSet::All, // TutorialChapter5 and all Challenges use all ghosts
        }
    }
}
