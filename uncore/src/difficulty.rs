//! ## Difficulty Module
//!
//! This module defines the difficulty system for the Unhaunter game, enabling
//! customization of various gameplay parameters to provide a range of challenges
//! for players.
//!
//! The `Difficulty` enum represents the different difficulty levels, each with a
//! unique name and associated settings.  The `DifficultyStruct` struct holds the
//! concrete values for a specific difficulty level, which are assembled using
//! methods within the `Difficulty` enum.
//!
//! By defining these settings directly within the `Difficulty` enum, you can
//! fine-tune the game experience for each difficulty level, providing a tailored
//! challenge for players.
use crate::celsius_to_kelvin;
use crate::components::truck_ui::TabContents;
use crate::types::gear_kind::{GearKind, PlayerGearKind};
use crate::types::ghost::definitions::GhostSet;
use crate::types::manual::ManualChapterIndex;
use bevy::prelude::Resource;
use enum_iterator::{Sequence, all};
use serde::{Deserialize, Serialize};

/// Represents the different difficulty levels for the Unhaunter game.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Sequence, Serialize, Deserialize, Default)]
pub enum Difficulty {
    #[default]
    TutorialChapter1, // Formerly NoviceInvestigator
    TutorialChapter2, // Formerly AdeptInvestigator
    TutorialChapter3, // Formerly SeniorInvestigator
    TutorialChapter4, // Formerly ExpertInvestigator
    TutorialChapter5, // Formerly AdeptSpecialist

    StandardChallenge, // Formerly LeadSpecialist
    HardChallenge,     // Formerly MasterSpecialist
    ExpertChallenge,   // Formerly ExpertOccultist
    MasterChallenge,   // Formerly MasterGuardian
}

impl Difficulty {
    /// Returns an iterator over all difficulty levels.
    pub fn all() -> impl Iterator<Item = Difficulty> {
        all().filter(|x: &Difficulty| x.is_enabled())
    }

    /// Returns the next difficulty level, wrapping around to the beginning if at the
    /// end.
    pub fn next(&self) -> Self {
        enum_iterator::next_cycle(self)
    }

    /// Returns the previous difficulty level, wrapping around to the end if at the
    /// beginning.
    pub fn prev(&self) -> Self {
        enum_iterator::previous_cycle(self)
    }

    pub fn is_enabled(&self) -> bool {
        matches!(
            self,
            Difficulty::TutorialChapter1
                | Difficulty::TutorialChapter2
                | Difficulty::TutorialChapter3
                | Difficulty::TutorialChapter4
                | Difficulty::TutorialChapter5
                | Difficulty::StandardChallenge
                | Difficulty::HardChallenge
                | Difficulty::ExpertChallenge
                | Difficulty::MasterChallenge
        )
    }

    pub fn is_tutorial_difficulty(&self) -> bool {
        matches!(
            self,
            Difficulty::TutorialChapter1
                | Difficulty::TutorialChapter2
                | Difficulty::TutorialChapter3
                | Difficulty::TutorialChapter4
                | Difficulty::TutorialChapter5
        )
    }

    // --- Ghost Behavior ---
    /// Returns the ghost's movement speed multiplier.
    ///
    /// A higher value indicates a faster ghost.
    pub fn ghost_speed(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.0,
            Difficulty::TutorialChapter2 => 1.05,
            Difficulty::TutorialChapter3 => 1.1,
            Difficulty::TutorialChapter4 => 1.15,
            Difficulty::TutorialChapter5 => 1.2,
            Difficulty::StandardChallenge => 1.3, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.5,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.8,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 2.5,   // Was MasterGuardian
        }
    }

    /// Returns the ghost's rage buildup multiplier.
    ///
    /// A higher value means the ghost becomes enraged more quickly.
    pub fn ghost_rage_likelihood(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.3,
            Difficulty::TutorialChapter2 => 1.3,
            Difficulty::TutorialChapter3 => 1.3,
            Difficulty::TutorialChapter4 => 1.3,
            Difficulty::TutorialChapter5 => 1.3,
            Difficulty::StandardChallenge => 1.3, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.6,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 2.8,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 4.0,   // Was MasterGuardian
        }
    }

    /// Returns the ghost's movement speed multiplier during a hunt.
    ///
    /// A higher value results in more aggressive pursuit of the player.
    pub fn ghost_hunting_aggression(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.1,
            Difficulty::TutorialChapter2 => 1.1,
            Difficulty::TutorialChapter3 => 1.1,
            Difficulty::TutorialChapter4 => 1.1,
            Difficulty::TutorialChapter5 => 1.2,
            Difficulty::StandardChallenge => 1.25, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.35,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.6,    // Was ExpertOccultist
            Difficulty::MasterChallenge => 2.6,    // Was MasterGuardian
        }
    }

    /// Returns the multiplier for how often the ghost interacts with objects or
    /// triggers events.
    ///
    /// A higher value leads to more frequent paranormal activity.
    pub fn ghost_interaction_frequency(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.9,
            Difficulty::TutorialChapter2 => 0.9,
            Difficulty::TutorialChapter3 => 0.9,
            Difficulty::TutorialChapter4 => 1.0,
            Difficulty::TutorialChapter5 => 1.0,
            Difficulty::StandardChallenge => 1.0, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.1,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.1,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 1.1,   // Was MasterGuardian
        }
    }

    /// Returns the multiplier for the duration of a ghost's hunting phase.
    ///
    /// A higher value results in longer hunts.
    pub fn ghost_hunt_duration(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.5,
            Difficulty::TutorialChapter2 => 0.7,
            Difficulty::TutorialChapter3 => 0.9,
            Difficulty::TutorialChapter4 => 1.0,
            Difficulty::TutorialChapter5 => 1.1,
            Difficulty::StandardChallenge => 1.15, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.25,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.4,    // Was ExpertOccultist
            Difficulty::MasterChallenge => 1.9,    // Was MasterGuardian
        }
    }

    /// Returns the multiplier for the time between ghost hunts.
    ///
    /// A higher value means longer periods of calm between hunts.
    pub fn ghost_hunt_cooldown(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 6.0,
            Difficulty::TutorialChapter2 => 5.0,
            Difficulty::TutorialChapter3 => 4.0,
            Difficulty::TutorialChapter4 => 3.0,
            Difficulty::TutorialChapter5 => 2.5,
            Difficulty::StandardChallenge => 2.0, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.0,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.7,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.05,  // Was MasterGuardian
        }
    }

    /// Returns the ghost's attraction factor to its breach (spawn point).
    ///
    /// A higher value means the ghost tends to stay closer to its breach, while a
    /// lower value allows the ghost to roam more freely.
    pub fn ghost_attraction_to_breach(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 10.0,
            Difficulty::TutorialChapter2 => 8.0,
            Difficulty::TutorialChapter3 => 5.0,
            Difficulty::TutorialChapter4 => 3.0,
            Difficulty::TutorialChapter5 => 1.5,
            Difficulty::StandardChallenge => 1.0, // Was LeadSpecialist
            Difficulty::HardChallenge => 0.7,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.5,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.05,  // Was MasterGuardian
        }
    }

    /// Returns the radius around the ghost's breach within which a Repulsive object
    /// can provoke a hunt.
    pub fn hunt_provocation_radius(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.5,
            Difficulty::TutorialChapter2 => 1.6,
            Difficulty::TutorialChapter3 => 1.7,
            Difficulty::TutorialChapter4 => 1.8,
            Difficulty::TutorialChapter5 => 1.9,
            Difficulty::StandardChallenge => 2.0, // Was LeadSpecialist
            Difficulty::HardChallenge => 2.4,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 3.0,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 5.0,   // Was MasterGuardian
        }
    }

    /// Returns the rate at which the ghost's anger increases when an attractive object
    /// is removed from the location.
    pub fn attractive_removal_anger_rate(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.02,
            Difficulty::TutorialChapter2 => 0.03,
            Difficulty::TutorialChapter3 => 0.04,
            Difficulty::TutorialChapter4 => 0.05,
            Difficulty::TutorialChapter5 => 0.06,
            Difficulty::StandardChallenge => 0.07, // Was LeadSpecialist
            Difficulty::HardChallenge => 0.09,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.12,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.2,    // Was MasterGuardian
        }
    }

    // --- Environment ---
    /// Returns the base ambient temperature of the location.
    pub fn ambient_temperature(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => celsius_to_kelvin(19.0),
            Difficulty::TutorialChapter2 => celsius_to_kelvin(18.5),
            Difficulty::TutorialChapter3 => celsius_to_kelvin(18.0),
            Difficulty::TutorialChapter4 => celsius_to_kelvin(17.0),
            Difficulty::TutorialChapter5 => celsius_to_kelvin(16.0),
            Difficulty::StandardChallenge => celsius_to_kelvin(15.0), // Was LeadSpecialist
            Difficulty::HardChallenge => celsius_to_kelvin(13.0),     // Was MasterSpecialist
            Difficulty::ExpertChallenge => celsius_to_kelvin(10.0),   // Was ExpertOccultist
            Difficulty::MasterChallenge => celsius_to_kelvin(6.0),    // Was MasterGuardian
        }
    }

    /// Returns the multiplier for how quickly temperature changes propagate through
    /// the environment.
    pub fn temperature_spread_speed(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 3.0,
            Difficulty::TutorialChapter2 => 2.8,
            Difficulty::TutorialChapter3 => 2.5,
            Difficulty::TutorialChapter4 => 2.3,
            Difficulty::TutorialChapter5 => 2.0,
            Difficulty::StandardChallenge => 1.5, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.0,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.9,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.65,  // Was MasterGuardian
        }
    }

    /// Returns the heat generation multiplier for light sources.
    ///
    /// A higher value means lights emit more heat.
    pub fn light_heat(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.1,
            Difficulty::TutorialChapter2 => 0.2,
            Difficulty::TutorialChapter3 => 0.3,
            Difficulty::TutorialChapter4 => 0.4,
            Difficulty::TutorialChapter5 => 0.6,
            Difficulty::StandardChallenge => 0.8, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.0,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.2,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 1.4,   // Was MasterGuardian
        }
    }

    /// Returns the multiplier for the intensity of darkness at low light levels.
    ///
    /// This simulates the eye's adaptation to darkness, with higher values resulting
    /// in a darker environment overall.
    pub fn darkness_intensity(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.6,
            Difficulty::TutorialChapter2 => 0.65,
            Difficulty::TutorialChapter3 => 0.7,
            Difficulty::TutorialChapter4 => 0.75,
            Difficulty::TutorialChapter5 => 0.9,
            Difficulty::StandardChallenge => 1.00, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.15,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.3,    // Was ExpertOccultist
            Difficulty::MasterChallenge => 1.6,    // Was MasterGuardian
        }
    }

    /// Returns the gamma correction value for the game environment.
    ///
    /// Higher values make the lighting appear harsher and less realistic, potentially
    /// highlighting details but creating a less atmospheric look.
    pub fn environment_gamma(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 2.5,
            Difficulty::TutorialChapter2 => 2.4,
            Difficulty::TutorialChapter3 => 2.3,
            Difficulty::TutorialChapter4 => 2.2,
            Difficulty::TutorialChapter5 => 2.0,
            Difficulty::StandardChallenge => 1.7, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.1,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.1, // Was ExpertOccultist (Kept from old ExpertOccultist)
            Difficulty::MasterChallenge => 0.6, // Was MasterGuardian
        }
    }

    // --- Player ---
    /// Returns the player's starting sanity level.
    pub fn max_recoverable_sanity(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 100.0,
            Difficulty::TutorialChapter2 => 100.0,
            Difficulty::TutorialChapter3 => 100.0,
            Difficulty::TutorialChapter4 => 100.0,
            Difficulty::TutorialChapter5 => 100.0,
            Difficulty::StandardChallenge => 100.0, // Was LeadSpecialist
            Difficulty::HardChallenge => 100.0,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 70.0,    // Was ExpertOccultist
            Difficulty::MasterChallenge => 60.0,    // Was MasterGuardian
        }
    }

    /// Returns the sanity drain rate multiplier.
    ///
    /// A higher value results in faster sanity loss for the player.
    pub fn sanity_drain_rate(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.1,
            Difficulty::TutorialChapter2 => 0.5,
            Difficulty::TutorialChapter3 => 0.7,
            Difficulty::TutorialChapter4 => 1.0,
            Difficulty::TutorialChapter5 => 1.5,
            Difficulty::StandardChallenge => 2.0, // Was LeadSpecialist
            Difficulty::HardChallenge => 4.0,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 10.0,  // Was ExpertOccultist
            Difficulty::MasterChallenge => 100.0, // Was MasterGuardian
        }
    }

    /// Returns the health drain rate multiplier during a ghost hunt.
    ///
    /// A higher value means the player loses health more quickly during hunts.
    pub fn health_drain_rate(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 0.4,
            Difficulty::TutorialChapter2 => 0.7,
            Difficulty::TutorialChapter3 => 0.9,
            Difficulty::TutorialChapter4 => 1.0,
            Difficulty::TutorialChapter5 => 1.05,
            Difficulty::StandardChallenge => 1.10, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.20,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.35,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 1.6,    // Was MasterGuardian
        }
    }

    /// Returns the health recovery rate multiplier.
    ///
    /// A higher value means the player regains health more quickly.
    pub fn health_recovery_rate(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.2,
            Difficulty::TutorialChapter2 => 1.1,
            Difficulty::TutorialChapter3 => 1.0,
            Difficulty::TutorialChapter4 => 0.95,
            Difficulty::TutorialChapter5 => 0.9,
            Difficulty::StandardChallenge => 0.85, // Was LeadSpecialist
            Difficulty::HardChallenge => 0.75,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.6,    // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.3,    // Was MasterGuardian
        }
    }

    /// Returns the player's movement speed multiplier.
    pub fn player_speed(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.3,
            Difficulty::TutorialChapter2 => 1.25,
            Difficulty::TutorialChapter3 => 1.20,
            Difficulty::TutorialChapter4 => 1.15,
            Difficulty::TutorialChapter5 => 1.1,
            Difficulty::StandardChallenge => 1.05, // Was LeadSpecialist
            Difficulty::HardChallenge => 1.0,      // Was MasterSpecialist
            Difficulty::ExpertChallenge => 1.0,    // Was ExpertOccultist
            Difficulty::MasterChallenge => 1.0,    // Was MasterGuardian
        }
    }

    // --- Evidence Gathering ---
    /// Returns the multiplier for the clarity or intensity of evidence manifestations.
    ///
    /// A higher value makes evidence more visible or noticeable.
    pub fn evidence_visibility(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.9,
            Difficulty::TutorialChapter2 => 1.7,
            Difficulty::TutorialChapter3 => 1.5,
            Difficulty::TutorialChapter4 => 1.2,
            Difficulty::TutorialChapter5 => 1.0,
            Difficulty::StandardChallenge => 0.9, // Was LeadSpecialist
            Difficulty::HardChallenge => 0.7,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.5,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.5,   // Was MasterGuardian
        }
    }

    /// Returns the sensitivity multiplier for the player's equipment.
    ///
    /// A higher value makes the equipment more responsive to paranormal activity.
    pub fn equipment_sensitivity(&self) -> f32 {
        match self {
            Difficulty::TutorialChapter1 => 1.3,
            Difficulty::TutorialChapter2 => 1.2,
            Difficulty::TutorialChapter3 => 1.1,
            Difficulty::TutorialChapter4 => 1.05,
            Difficulty::TutorialChapter5 => 1.0,
            Difficulty::StandardChallenge => 0.95, // Was LeadSpecialist
            Difficulty::HardChallenge => 0.85,     // Was MasterSpecialist
            Difficulty::ExpertChallenge => 0.78,   // Was ExpertOccultist
            Difficulty::MasterChallenge => 0.65,   // Was MasterGuardian
        }
    }

    // --- Gameplay ---
    /// Returns a boolean value indicating whether the van UI automatically opens at
    /// the beginning of a mission.
    pub fn van_auto_open(&self) -> bool {
        !matches!(
            self,
            Difficulty::TutorialChapter1 | Difficulty::TutorialChapter2
        )
    }

    /// Returns the default tab selected in the van UI.
    pub fn default_van_tab(&self) -> TabContents {
        match self {
            Difficulty::TutorialChapter1 | Difficulty::TutorialChapter2 => TabContents::Journal,
            _ => TabContents::Loadout,
        }
    }

    pub fn player_gear(&self) -> PlayerGearKind {
        match self {
            Difficulty::TutorialChapter1 => PlayerGearKind {
                left_hand: GearKind::Flashlight,
                right_hand: GearKind::Thermometer,
                inventory: vec![GearKind::EMFMeter, GearKind::None],
            },
            Difficulty::TutorialChapter2 => PlayerGearKind {
                left_hand: GearKind::Videocam,
                right_hand: GearKind::UVTorch,
                inventory: vec![GearKind::Thermometer, GearKind::EMFMeter],
            },
            _ => PlayerGearKind {
                left_hand: GearKind::Flashlight,
                right_hand: GearKind::None,
                inventory: vec![GearKind::None, GearKind::None],
            },
        }
    }

    pub fn ghost_set(&self) -> GhostSet {
        match self {
            Difficulty::TutorialChapter1 => GhostSet::TmpEMF,
            Difficulty::TutorialChapter2 => GhostSet::TmpEMFUVOrbs,
            Difficulty::TutorialChapter3 => GhostSet::TmpEMFUVOrbsEVPCPM,
            Difficulty::TutorialChapter4 => GhostSet::Twenty,
            _ => GhostSet::All, // TutorialChapter5 and all Challenges use all ghosts
        }
    }

    pub fn truck_gear(&self) -> Vec<GearKind> {
        use crate::types::gear_kind::GearKind::*;
        let mut gear = Vec::new();

        match self {
            Difficulty::TutorialChapter1 => {
                gear.push(Flashlight);
                gear.push(Thermometer);
                gear.push(EMFMeter);
            }
            Difficulty::TutorialChapter2 => {
                gear.extend(Self::TutorialChapter1.truck_gear());
                gear.push(UVTorch);
                gear.push(Videocam);
            }
            Difficulty::TutorialChapter3 => {
                gear.extend(Self::TutorialChapter2.truck_gear());
                gear.push(Recorder);
                gear.push(GeigerCounter);
            }
            Difficulty::TutorialChapter4 => {
                gear.extend(Self::TutorialChapter3.truck_gear());
                gear.push(SpiritBox);
                gear.push(RedTorch);
            }
            Difficulty::TutorialChapter5 => {
                gear.extend(Self::TutorialChapter4.truck_gear());
                gear.push(Salt);
                gear.push(QuartzStone);
                gear.push(SageBundle);
            }
            // For StandardChallenge and above, they get all gear from TutorialChapter5
            Difficulty::StandardChallenge
            | Difficulty::HardChallenge
            | Difficulty::ExpertChallenge
            | Difficulty::MasterChallenge => {
                gear = Self::TutorialChapter5.truck_gear();
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

    // --- UI and Scoring ---
    /// Returns the display name for the difficulty level.
    pub fn difficulty_name(&self) -> &'static str {
        match self {
            Difficulty::TutorialChapter1 => "Tutorial: Chapter 1",
            Difficulty::TutorialChapter2 => "Tutorial: Chapter 2",
            Difficulty::TutorialChapter3 => "Tutorial: Chapter 3",
            Difficulty::TutorialChapter4 => "Tutorial: Chapter 4",
            Difficulty::TutorialChapter5 => "Tutorial: Chapter 5",
            Difficulty::StandardChallenge => "Standard Challenge",
            Difficulty::HardChallenge => "Hard Challenge",
            Difficulty::ExpertChallenge => "Expert Challenge",
            Difficulty::MasterChallenge => "Master Challenge",
        }
    }

    pub fn difficulty_description(&self) -> &'static str {
        match self {
            Difficulty::TutorialChapter1 => {
                "Basics of investigation. Learn to use essential tools: Flashlight, Thermometer, and EMF Reader. Identify simple ghosts. Ideal for new players starting the campaign."
            }
            Difficulty::TutorialChapter2 => {
                "Continue your training. Add UV Torch and Video Camera to your toolkit. Learn to spot UV traces and visual evidence."
            }
            Difficulty::TutorialChapter3 => {
                "Advanced tools introduced: Voice Recorder and Geiger Counter. Face a wider variety of ghosts and gather more complex evidence."
            }
            Difficulty::TutorialChapter4 => {
                "Further expand your arsenal with the SpiritBox and Red Torch. Delve deeper into communication with the other side and object interactions."
            }
            Difficulty::TutorialChapter5 => {
                "Master protective and manipulative tools: Salt, Quartz Stone, and Sage Bundle. Handle more dangerous paranormal threats."
            }
            Difficulty::StandardChallenge => {
                "A balanced investigation experience. Ghosts are moderately active and challenging. All gear and ghost types available."
            }
            Difficulty::HardChallenge => {
                "For seasoned investigators. Ghosts are more aggressive, evidence can be trickier, and your resources might feel strained."
            }
            Difficulty::ExpertChallenge => {
                "A true test of skill. Ghosts are highly dangerous, sanity drains quickly, and the environment itself can be hostile."
            }
            Difficulty::MasterChallenge => {
                "Only for the most fearless and experienced. Expect relentless paranormal activity, extreme conditions, and a fight for survival."
            }
        }
    }

    /// Returns the score multiplier for the end-of-mission summary, based on the
    /// chosen difficulty level.
    ///
    /// Higher values reward players more for completing missions on harder
    /// difficulties.
    pub fn difficulty_score_multiplier(&self) -> f64 {
        match self {
            Difficulty::TutorialChapter1 => 1.0,
            Difficulty::TutorialChapter2 => 2.0,
            Difficulty::TutorialChapter3 => 4.0,
            Difficulty::TutorialChapter4 => 6.0,
            Difficulty::TutorialChapter5 => 8.0,
            Difficulty::StandardChallenge => 10.0,
            Difficulty::HardChallenge => 20.0,
            Difficulty::ExpertChallenge => 50.0,
            Difficulty::MasterChallenge => 100.0,
        }
    }

    pub fn tutorial_chapter(&self) -> Option<ManualChapterIndex> {
        match self {
            Difficulty::TutorialChapter1 => Some(ManualChapterIndex::Chapter1),
            Difficulty::TutorialChapter2 => Some(ManualChapterIndex::Chapter2),
            Difficulty::TutorialChapter3 => Some(ManualChapterIndex::Chapter3),
            Difficulty::TutorialChapter4 => Some(ManualChapterIndex::Chapter4),
            Difficulty::TutorialChapter5 => Some(ManualChapterIndex::Chapter5),
            _ => None,
        }
    }

    /// Creates a `DifficultyStruct` instance with the settings for the current
    /// difficulty level.
    ///
    /// This method aggregates all the individual parameter settings defined by the
    /// other methods in this enum.
    pub fn create_difficulty_struct(&self) -> DifficultyStruct {
        DifficultyStruct {
            ghost_speed: self.ghost_speed(),
            ghost_rage_likelihood: self.ghost_rage_likelihood(),
            ghost_hunting_aggression: self.ghost_hunting_aggression(),
            ghost_interaction_frequency: self.ghost_interaction_frequency(),
            ghost_hunt_duration: self.ghost_hunt_duration(),
            ghost_hunt_cooldown: self.ghost_hunt_cooldown(),
            ghost_attraction_to_breach: self.ghost_attraction_to_breach(),
            hunt_provocation_radius: self.hunt_provocation_radius(),
            attractive_removal_anger_rate: self.attractive_removal_anger_rate(),
            ambient_temperature: self.ambient_temperature(),
            temperature_spread_speed: self.temperature_spread_speed(),
            light_heat: self.light_heat(),
            darkness_intensity: self.darkness_intensity(),
            environment_gamma: self.environment_gamma(),
            max_recoverable_sanity: self.max_recoverable_sanity(),
            sanity_drain_rate: self.sanity_drain_rate(),
            health_drain_rate: self.health_drain_rate(),
            health_recovery_rate: self.health_recovery_rate(),
            player_speed: self.player_speed(),
            evidence_visibility: self.evidence_visibility(),
            equipment_sensitivity: self.equipment_sensitivity(),
            van_auto_open: self.van_auto_open(),
            default_van_tab: self.default_van_tab(),
            player_gear: self.player_gear(),
            ghost_set: self.ghost_set(),
            difficulty: *self,
            difficulty_name: self.difficulty_name().to_string(),
            difficulty_description: self.difficulty_description().to_owned(),
            difficulty_score_multiplier: self.difficulty_score_multiplier(),
            tutorial_chapter: self.tutorial_chapter(),
            truck_gear: self.truck_gear(),
        }
    }
}

/// Holds the concrete values for a specific difficulty level.
///
/// This struct is assembled by the `create_difficulty_struct` method in the
/// `Difficulty` enum.
#[derive(Debug, Clone, Serialize, Deserialize, Resource, PartialEq)]
pub struct DifficultyStruct {
    // --- Ghost Behavior ---
    pub ghost_speed: f32,
    pub ghost_rage_likelihood: f32,
    pub ghost_hunting_aggression: f32,
    pub ghost_interaction_frequency: f32,
    pub ghost_hunt_duration: f32,
    pub ghost_hunt_cooldown: f32,
    pub ghost_attraction_to_breach: f32,
    /// The radius around the ghost's breach within which a Repulsive object can
    /// provoke a hunt.
    pub hunt_provocation_radius: f32,
    /// The rate at which the ghost's anger increases when an Attractive object is
    /// removed from the location.
    pub attractive_removal_anger_rate: f32,
    // --- Environment ---
    pub ambient_temperature: f32,
    pub temperature_spread_speed: f32,
    pub light_heat: f32,
    pub darkness_intensity: f32,
    pub environment_gamma: f32,
    // --- Player ---
    pub max_recoverable_sanity: f32,
    pub sanity_drain_rate: f32,
    pub health_drain_rate: f32,
    pub health_recovery_rate: f32,
    pub player_speed: f32,
    // --- Evidence Gathering ---
    pub evidence_visibility: f32,
    pub equipment_sensitivity: f32,
    // --- Gameplay ---
    pub van_auto_open: bool,
    pub default_van_tab: TabContents,
    pub player_gear: PlayerGearKind,
    pub ghost_set: GhostSet,
    // --- UI and Scoring ---
    pub difficulty_name: String,
    pub difficulty: Difficulty,
    pub difficulty_description: String,
    pub difficulty_score_multiplier: f64,
    /// The range of manual pages associated with this difficulty.
    pub tutorial_chapter: Option<ManualChapterIndex>,
    pub truck_gear: Vec<GearKind>,
}

impl Default for DifficultyStruct {
    fn default() -> Self {
        Difficulty::default().create_difficulty_struct()
    }
}

/// Represents the currently selected difficulty level.
///
/// This resource is used to store the difficulty settings that are currently
/// active in the game. It is updated when the player changes the difficulty
/// in the game settings or when a new mission with a specific difficulty is
/// loaded.
#[derive(Debug, Clone, Serialize, Deserialize, Resource, PartialEq, Default)]
pub struct CurrentDifficulty(pub DifficultyStruct);

impl CurrentDifficulty {
    /// Creates a new `CurrentDifficulty` resource with the specified difficulty
    /// level.
    pub fn new(difficulty: Difficulty) -> Self {
        CurrentDifficulty(difficulty.create_difficulty_struct())
    }
}

/// Returns the `DifficultyStruct` for the specified difficulty level.
pub fn get_difficulty_struct(difficulty: Difficulty) -> DifficultyStruct {
    difficulty.create_difficulty_struct()
}
