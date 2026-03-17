//! Extension trait for Difficulty to access gear-related settings
//!
//! This trait breaks the dependency inversion: instead of undifficulty-core depending on ungear-core,
//! ungear-core now provides this extension trait to define what gear is available for each difficulty.

use crate::types::gear::kind::{GearKind, PlayerGearKind};
use untypes_core::difficulty::Difficulty;

/// Extension trait providing gear-related difficulty settings.
///
/// This trait is implemented by `Difficulty` to provide loadout information without creating a
/// dependency from undifficulty-core to ungear-core.
pub trait DifficultyGearExt {
    /// Returns the truck gear available for this difficulty.
    fn truck_gear(&self) -> Vec<GearKind>;

    /// Returns the player's starting gear for this difficulty.
    fn player_gear(&self) -> PlayerGearKind;
}

impl DifficultyGearExt for Difficulty {
    fn truck_gear(&self) -> Vec<GearKind> {
        use crate::types::gear::kind::GearKind::*;
        let mut gear = Vec::new();

        match self {
            Difficulty::TutorialChapter1 => {
                gear.push(Flashlight);
                gear.push(Thermometer);
                gear.push(EMFMeter);
            }
            Difficulty::TutorialChapter2 => {
                gear.extend(Difficulty::TutorialChapter1.truck_gear());
                gear.push(UVTorch);
                gear.push(Videocam);
            }
            Difficulty::TutorialChapter3 => {
                gear.extend(Difficulty::TutorialChapter2.truck_gear());
                gear.push(Recorder);
                gear.push(GeigerCounter);
            }
            Difficulty::TutorialChapter4 => {
                gear.extend(Difficulty::TutorialChapter3.truck_gear());
                gear.push(SpiritBox);
                gear.push(RedTorch);
            }
            Difficulty::TutorialChapter5 => {
                gear.extend(Difficulty::TutorialChapter4.truck_gear());
                gear.push(Salt);
                gear.push(QuartzStone);
                gear.push(SageBundle);
            }
            // For StandardChallenge and above, they get all gear from TutorialChapter5
            Difficulty::StandardChallenge
            | Difficulty::HardChallenge
            | Difficulty::ExpertChallenge
            | Difficulty::MasterChallenge => {
                gear = Difficulty::TutorialChapter5.truck_gear();
            }
        }

        // This is for debugging purposes, to add gear that isn't functional yet.
        const ENABLE_INCOMPLETE: bool = false;
        if ENABLE_INCOMPLETE {
            let mut incomplete: Vec<GearKind> = vec![
                IonMeter,
                ThermalImager,
                Photocam,
                Compass,
                EStaticMeter,
                MotionSensor,
            ];
            gear.append(&mut incomplete);
        }
        gear
    }

    fn player_gear(&self) -> PlayerGearKind {
        match self {
            Difficulty::TutorialChapter1 => PlayerGearKind {
                left_hand: GearKind::Flashlight,
                right_hand: GearKind::Thermometer,
                inventory: vec![GearKind::EMFMeter, GearKind::None],
            },
            Difficulty::TutorialChapter2 => PlayerGearKind {
                left_hand: GearKind::UVTorch,
                right_hand: GearKind::Thermometer,
                inventory: vec![GearKind::Videocam, GearKind::EMFMeter],
            },
            _ => PlayerGearKind {
                left_hand: GearKind::Flashlight,
                right_hand: GearKind::None,
                inventory: vec![GearKind::None, GearKind::None],
            },
        }
    }
}
