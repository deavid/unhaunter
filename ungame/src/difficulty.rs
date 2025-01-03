// src/difficulty.rs
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
use bevy::prelude::Resource;
use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};
use uncore::components::truck_ui::TabContents;
use uncore::types::gear_kind::{GearKind, PlayerGearKind};
use uncore::types::ghost::definitions::GhostSet;
use uncore::types::manual::ManualChapter;

/// Represents the different difficulty levels for the Unhaunter game.
///
/// Each variant corresponds to a specific difficulty preset with unique settings,
/// ranging from `Apprentice` (easiest) to `Legend` (hardest).
///
/// These difficulty levels impact various gameplay parameters, including ghost
/// behavior, environment conditions, player attributes, and general gameplay
/// mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Serialize, Deserialize, Default)]
pub enum Difficulty {
    #[default]
    NoviceInvestigator,
    AdeptInvestigator,
    SeniorInvestigator,
    ExpertInvestigator,
    AdeptSpecialist,
    LeadSpecialist,
    ExpertSpecialist,
    MasterSpecialist,
    InitiateOccultist,
    AdeptOccultist,
    ExpertOccultist,
    MasterOccultist,
    AdeptGuardian,
    LeadGuardian,
    ExpertGuardian,
    MasterGuardian,
}

impl Difficulty {
    /// Returns an iterator over all difficulty levels.
    pub fn all() -> enum_iterator::All<Self> {
        all()
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

    // --- Ghost Behavior ---
    /// Returns the ghost's movement speed multiplier.
    ///
    /// A higher value indicates a faster ghost.
    pub fn ghost_speed(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.4,
            Difficulty::AdeptInvestigator => 0.6,
            Difficulty::SeniorInvestigator => 0.8,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.2,
            Difficulty::LeadSpecialist => 1.3,
            Difficulty::ExpertSpecialist => 1.4,
            Difficulty::MasterSpecialist => 1.5,
            Difficulty::InitiateOccultist => 1.6,
            Difficulty::AdeptOccultist => 1.7,
            Difficulty::ExpertOccultist => 1.8,
            Difficulty::MasterOccultist => 1.9,
            Difficulty::AdeptGuardian => 2.0,
            Difficulty::LeadGuardian => 2.1,
            Difficulty::ExpertGuardian => 2.2,
            Difficulty::MasterGuardian => 2.5,
        }
    }

    /// Returns the ghost's rage buildup multiplier.
    ///
    /// A higher value means the ghost becomes enraged more quickly.
    pub fn ghost_rage_likelihood(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 1.0,
            Difficulty::AdeptInvestigator => 1.0,
            Difficulty::SeniorInvestigator => 1.0,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.0,
            Difficulty::LeadSpecialist => 1.2,
            Difficulty::ExpertSpecialist => 1.4,
            Difficulty::MasterSpecialist => 1.6,
            Difficulty::InitiateOccultist => 2.0,
            Difficulty::AdeptOccultist => 2.4,
            Difficulty::ExpertOccultist => 2.8,
            Difficulty::MasterOccultist => 3.2,
            Difficulty::AdeptGuardian => 3.4,
            Difficulty::LeadGuardian => 3.6,
            Difficulty::ExpertGuardian => 3.8,
            Difficulty::MasterGuardian => 4.0,
        }
    }

    /// Returns the ghost's movement speed multiplier during a hunt.
    ///
    /// A higher value results in more aggressive pursuit of the player.
    pub fn ghost_hunting_aggression(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.2,
            Difficulty::AdeptInvestigator => 0.4,
            Difficulty::SeniorInvestigator => 0.8,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.2,
            Difficulty::LeadSpecialist => 1.25,
            Difficulty::ExpertSpecialist => 1.3,
            Difficulty::MasterSpecialist => 1.35,
            Difficulty::InitiateOccultist => 1.5,
            Difficulty::AdeptOccultist => 1.55,
            Difficulty::ExpertOccultist => 1.6,
            Difficulty::MasterOccultist => 1.8,
            Difficulty::AdeptGuardian => 2.0,
            Difficulty::LeadGuardian => 2.2,
            Difficulty::ExpertGuardian => 2.4,
            Difficulty::MasterGuardian => 2.6,
        }
    }

    /// Returns the multiplier for how often the ghost interacts with objects or
    /// triggers events.
    ///
    /// A higher value leads to more frequent paranormal activity.
    pub fn ghost_interaction_frequency(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.5,
            Difficulty::AdeptInvestigator => 0.7,
            Difficulty::SeniorInvestigator => 0.9,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.1,
            Difficulty::LeadSpecialist => 1.2,
            Difficulty::ExpertSpecialist => 1.3,
            Difficulty::MasterSpecialist => 1.4,
            Difficulty::InitiateOccultist => 1.5,
            Difficulty::AdeptOccultist => 1.6,
            Difficulty::ExpertOccultist => 1.7,
            Difficulty::MasterOccultist => 1.8,
            Difficulty::AdeptGuardian => 1.9,
            Difficulty::LeadGuardian => 2.0,
            Difficulty::ExpertGuardian => 2.0,
            Difficulty::MasterGuardian => 2.0,
        }
    }

    /// Returns the multiplier for the duration of a ghost's hunting phase.
    ///
    /// A higher value results in longer hunts.
    pub fn ghost_hunt_duration(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.1,
            Difficulty::AdeptInvestigator => 0.3,
            Difficulty::SeniorInvestigator => 0.7,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.1,
            Difficulty::LeadSpecialist => 1.2,
            Difficulty::ExpertSpecialist => 1.4,
            Difficulty::MasterSpecialist => 1.6,
            Difficulty::InitiateOccultist => 2.0,
            Difficulty::AdeptOccultist => 2.5,
            Difficulty::ExpertOccultist => 3.0,
            Difficulty::MasterOccultist => 3.5,
            Difficulty::AdeptGuardian => 4.0,
            Difficulty::LeadGuardian => 4.5,
            Difficulty::ExpertGuardian => 5.0,
            Difficulty::MasterGuardian => 6.0,
        }
    }

    /// Returns the multiplier for the time between ghost hunts.
    ///
    /// A higher value means longer periods of calm between hunts.
    pub fn ghost_hunt_cooldown(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 3.0,
            Difficulty::AdeptInvestigator => 3.0,
            Difficulty::SeniorInvestigator => 3.0,
            Difficulty::ExpertInvestigator => 3.0,
            Difficulty::AdeptSpecialist => 2.5,
            Difficulty::LeadSpecialist => 2.0,
            Difficulty::ExpertSpecialist => 1.5,
            Difficulty::MasterSpecialist => 1.0,
            Difficulty::InitiateOccultist => 1.0,
            Difficulty::AdeptOccultist => 1.0,
            Difficulty::ExpertOccultist => 1.0,
            Difficulty::MasterOccultist => 1.0,
            Difficulty::AdeptGuardian => 0.6,
            Difficulty::LeadGuardian => 0.3,
            Difficulty::ExpertGuardian => 0.1,
            Difficulty::MasterGuardian => 0.02,
        }
    }

    /// Returns the ghost's attraction factor to its breach (spawn point).
    ///
    /// A higher value means the ghost tends to stay closer to its breach, while a
    /// lower value allows the ghost to roam more freely.
    pub fn ghost_attraction_to_breach(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 10.0,
            Difficulty::AdeptInvestigator => 8.0,
            Difficulty::SeniorInvestigator => 5.0,
            Difficulty::ExpertInvestigator => 3.0,
            Difficulty::AdeptSpecialist => 1.5,
            Difficulty::LeadSpecialist => 1.0,
            Difficulty::ExpertSpecialist => 0.8,
            Difficulty::MasterSpecialist => 0.7,
            Difficulty::InitiateOccultist => 0.6,
            Difficulty::AdeptOccultist => 0.55,
            Difficulty::ExpertOccultist => 0.5,
            Difficulty::MasterOccultist => 0.45,
            Difficulty::AdeptGuardian => 0.4,
            Difficulty::LeadGuardian => 0.2,
            Difficulty::ExpertGuardian => 0.1,
            Difficulty::MasterGuardian => 0.05,
        }
    }

    /// Returns the radius around the ghost's breach within which a Repulsive object
    /// can provoke a hunt.
    pub fn hunt_provocation_radius(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.3,
            Difficulty::AdeptInvestigator => 0.35,
            Difficulty::SeniorInvestigator => 0.4,
            Difficulty::ExpertInvestigator => 0.45,
            Difficulty::AdeptSpecialist => 0.6,
            Difficulty::LeadSpecialist => 0.7,
            Difficulty::ExpertSpecialist => 0.9,
            Difficulty::MasterSpecialist => 1.0,
            Difficulty::InitiateOccultist => 1.2,
            Difficulty::AdeptOccultist => 1.3,
            Difficulty::ExpertOccultist => 1.4,
            Difficulty::MasterOccultist => 1.5,
            Difficulty::AdeptGuardian => 1.6,
            Difficulty::LeadGuardian => 1.7,
            Difficulty::ExpertGuardian => 1.8,
            Difficulty::MasterGuardian => 2.0,
        }
    }

    /// Returns the rate at which the ghost's anger increases when an attractive object
    /// is removed from the location.
    pub fn attractive_removal_anger_rate(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.02,
            Difficulty::AdeptInvestigator => 0.03,
            Difficulty::SeniorInvestigator => 0.04,
            Difficulty::ExpertInvestigator => 0.05,
            Difficulty::AdeptSpecialist => 0.06,
            Difficulty::LeadSpecialist => 0.07,
            Difficulty::ExpertSpecialist => 0.08,
            Difficulty::MasterSpecialist => 0.09,
            Difficulty::InitiateOccultist => 0.1,
            Difficulty::AdeptOccultist => 0.11,
            Difficulty::ExpertOccultist => 0.12,
            Difficulty::MasterOccultist => 0.13,
            Difficulty::AdeptGuardian => 0.14,
            Difficulty::LeadGuardian => 0.15,
            Difficulty::ExpertGuardian => 0.16,
            Difficulty::MasterGuardian => 0.2,
        }
    }

    // --- Environment ---
    /// Returns the base ambient temperature of the location.
    pub fn ambient_temperature(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 19.0,
            Difficulty::AdeptInvestigator => 18.0,
            Difficulty::SeniorInvestigator => 16.0,
            Difficulty::ExpertInvestigator => 14.0,
            Difficulty::AdeptSpecialist => 12.0,
            Difficulty::LeadSpecialist => 11.0,
            Difficulty::ExpertSpecialist => 10.0,
            Difficulty::MasterSpecialist => 9.0,
            Difficulty::InitiateOccultist => 8.0,
            Difficulty::AdeptOccultist => 7.0,
            Difficulty::ExpertOccultist => 6.0,
            Difficulty::MasterOccultist => 5.0,
            Difficulty::AdeptGuardian => 5.0,
            Difficulty::LeadGuardian => 4.5,
            Difficulty::ExpertGuardian => 4.0,
            Difficulty::MasterGuardian => 4.0,
        }
    }

    /// Returns the multiplier for how quickly temperature changes propagate through
    /// the environment.
    pub fn temperature_spread_speed(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 3.0,
            Difficulty::AdeptInvestigator => 2.8,
            Difficulty::SeniorInvestigator => 2.5,
            Difficulty::ExpertInvestigator => 2.3,
            Difficulty::AdeptSpecialist => 2.0,
            Difficulty::LeadSpecialist => 1.5,
            Difficulty::ExpertSpecialist => 1.2,
            Difficulty::MasterSpecialist => 1.0,
            Difficulty::InitiateOccultist => 1.0,
            Difficulty::AdeptOccultist => 0.9,
            Difficulty::ExpertOccultist => 0.8,
            Difficulty::MasterOccultist => 0.7,
            Difficulty::AdeptGuardian => 0.6,
            Difficulty::LeadGuardian => 0.4,
            Difficulty::ExpertGuardian => 0.3,
            Difficulty::MasterGuardian => 0.2,
        }
    }

    /// Returns the heat generation multiplier for light sources.
    ///
    /// A higher value means lights emit more heat.
    pub fn light_heat(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.1,
            Difficulty::AdeptInvestigator => 0.2,
            Difficulty::SeniorInvestigator => 0.3,
            Difficulty::ExpertInvestigator => 0.4,
            Difficulty::AdeptSpecialist => 0.6,
            Difficulty::LeadSpecialist => 0.8,
            Difficulty::ExpertSpecialist => 1.0,
            Difficulty::MasterSpecialist => 1.5,
            Difficulty::InitiateOccultist => 2.0,
            Difficulty::AdeptOccultist => 2.5,
            Difficulty::ExpertOccultist => 3.0,
            Difficulty::MasterOccultist => 3.5,
            Difficulty::AdeptGuardian => 4.0,
            Difficulty::LeadGuardian => 4.5,
            Difficulty::ExpertGuardian => 5.0,
            Difficulty::MasterGuardian => 5.0,
        }
    }

    /// Returns the multiplier for the intensity of darkness at low light levels.
    ///
    /// This simulates the eye's adaptation to darkness, with higher values resulting
    /// in a darker environment overall.
    pub fn darkness_intensity(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.6,
            Difficulty::AdeptInvestigator => 0.65,
            Difficulty::SeniorInvestigator => 0.7,
            Difficulty::ExpertInvestigator => 0.75,
            Difficulty::AdeptSpecialist => 0.9,
            Difficulty::LeadSpecialist => 1.00,
            Difficulty::ExpertSpecialist => 1.1,
            Difficulty::MasterSpecialist => 1.15,
            Difficulty::InitiateOccultist => 1.2,
            Difficulty::AdeptOccultist => 1.25,
            Difficulty::ExpertOccultist => 1.3,
            Difficulty::MasterOccultist => 1.35,
            Difficulty::AdeptGuardian => 1.4,
            Difficulty::LeadGuardian => 1.45,
            Difficulty::ExpertGuardian => 1.5,
            Difficulty::MasterGuardian => 1.6,
        }
    }

    /// Returns the gamma correction value for the game environment.
    ///
    /// Higher values make the lighting appear harsher and less realistic, potentially
    /// highlighting details but creating a less atmospheric look.
    pub fn environment_gamma(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 2.5,
            Difficulty::AdeptInvestigator => 2.4,
            Difficulty::SeniorInvestigator => 2.3,
            Difficulty::ExpertInvestigator => 2.2,
            Difficulty::AdeptSpecialist => 2.0,
            Difficulty::LeadSpecialist => 1.7,
            Difficulty::ExpertSpecialist => 1.4,
            Difficulty::MasterSpecialist => 1.1,
            Difficulty::InitiateOccultist => 1.1,
            Difficulty::AdeptOccultist => 1.1,
            Difficulty::ExpertOccultist => 1.1,
            Difficulty::MasterOccultist => 1.0,
            Difficulty::AdeptGuardian => 0.9,
            Difficulty::LeadGuardian => 0.8,
            Difficulty::ExpertGuardian => 0.7,
            Difficulty::MasterGuardian => 0.6,
        }
    }

    // --- Player ---
    /// Returns the player's starting sanity level.
    pub fn starting_sanity(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 100.0,
            Difficulty::AdeptInvestigator => 100.0,
            Difficulty::SeniorInvestigator => 100.0,
            Difficulty::ExpertInvestigator => 100.0,
            Difficulty::AdeptSpecialist => 100.0,
            Difficulty::LeadSpecialist => 100.0,
            Difficulty::ExpertSpecialist => 100.0,
            Difficulty::MasterSpecialist => 100.0,
            Difficulty::InitiateOccultist => 90.0,
            Difficulty::AdeptOccultist => 80.0,
            Difficulty::ExpertOccultist => 70.0,
            Difficulty::MasterOccultist => 60.0,
            Difficulty::AdeptGuardian => 60.0,
            Difficulty::LeadGuardian => 60.0,
            Difficulty::ExpertGuardian => 60.0,
            Difficulty::MasterGuardian => 60.0,
        }
    }

    /// Returns the sanity drain rate multiplier.
    ///
    /// A higher value results in faster sanity loss for the player.
    pub fn sanity_drain_rate(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.1,
            Difficulty::AdeptInvestigator => 0.5,
            Difficulty::SeniorInvestigator => 0.7,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.5,
            Difficulty::LeadSpecialist => 2.0,
            Difficulty::ExpertSpecialist => 3.0,
            Difficulty::MasterSpecialist => 4.0,
            Difficulty::InitiateOccultist => 6.0,
            Difficulty::AdeptOccultist => 8.0,
            Difficulty::ExpertOccultist => 10.0,
            Difficulty::MasterOccultist => 15.0,
            Difficulty::AdeptGuardian => 20.0,
            Difficulty::LeadGuardian => 30.0,
            Difficulty::ExpertGuardian => 50.0,
            Difficulty::MasterGuardian => 100.0,
        }
    }

    /// Returns the health drain rate multiplier during a ghost hunt.
    ///
    /// A higher value means the player loses health more quickly during hunts.
    pub fn health_drain_rate(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 0.4,
            Difficulty::AdeptInvestigator => 0.7,
            Difficulty::SeniorInvestigator => 0.9,
            Difficulty::ExpertInvestigator => 1.0,
            Difficulty::AdeptSpecialist => 1.05,
            Difficulty::LeadSpecialist => 1.10,
            Difficulty::ExpertSpecialist => 1.15,
            Difficulty::MasterSpecialist => 1.20,
            Difficulty::InitiateOccultist => 1.25,
            Difficulty::AdeptOccultist => 1.30,
            Difficulty::ExpertOccultist => 1.35,
            Difficulty::MasterOccultist => 1.40,
            Difficulty::AdeptGuardian => 1.45,
            Difficulty::LeadGuardian => 1.5,
            Difficulty::ExpertGuardian => 1.55,
            Difficulty::MasterGuardian => 1.6,
        }
    }

    /// Returns the health recovery rate multiplier.
    ///
    /// A higher value means the player regains health more quickly.
    pub fn health_recovery_rate(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 1.2,
            Difficulty::AdeptInvestigator => 1.1,
            Difficulty::SeniorInvestigator => 1.0,
            Difficulty::ExpertInvestigator => 0.95,
            Difficulty::AdeptSpecialist => 0.9,
            Difficulty::LeadSpecialist => 0.85,
            Difficulty::ExpertSpecialist => 0.8,
            Difficulty::MasterSpecialist => 0.75,
            Difficulty::InitiateOccultist => 0.7,
            Difficulty::AdeptOccultist => 0.65,
            Difficulty::ExpertOccultist => 0.6,
            Difficulty::MasterOccultist => 0.55,
            Difficulty::AdeptGuardian => 0.5,
            Difficulty::LeadGuardian => 0.45,
            Difficulty::ExpertGuardian => 0.4,
            Difficulty::MasterGuardian => 0.3,
        }
    }

    /// Returns the player's movement speed multiplier.
    pub fn player_speed(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 1.3,
            Difficulty::AdeptInvestigator => 1.25,
            Difficulty::SeniorInvestigator => 1.20,
            Difficulty::ExpertInvestigator => 1.15,
            Difficulty::AdeptSpecialist => 1.1,
            Difficulty::LeadSpecialist => 1.05,
            Difficulty::ExpertSpecialist => 1.00,
            Difficulty::MasterSpecialist => 0.98,
            Difficulty::InitiateOccultist => 0.95,
            Difficulty::AdeptOccultist => 0.92,
            Difficulty::ExpertOccultist => 0.90,
            Difficulty::MasterOccultist => 0.87,
            Difficulty::AdeptGuardian => 0.85,
            Difficulty::LeadGuardian => 0.83,
            Difficulty::ExpertGuardian => 0.82,
            Difficulty::MasterGuardian => 0.80,
        }
    }

    // --- Evidence Gathering ---
    /// Returns the multiplier for the clarity or intensity of evidence manifestations.
    ///
    /// A higher value makes evidence more visible or noticeable.
    pub fn evidence_visibility(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 1.9,
            Difficulty::AdeptInvestigator => 1.7,
            Difficulty::SeniorInvestigator => 1.5,
            Difficulty::ExpertInvestigator => 1.2,
            Difficulty::AdeptSpecialist => 1.0,
            Difficulty::LeadSpecialist => 0.9,
            Difficulty::ExpertSpecialist => 0.8,
            Difficulty::MasterSpecialist => 0.7,
            Difficulty::InitiateOccultist => 0.6,
            Difficulty::AdeptOccultist => 0.55,
            Difficulty::ExpertOccultist => 0.5,
            Difficulty::MasterOccultist => 0.5,
            Difficulty::AdeptGuardian => 0.5,
            Difficulty::LeadGuardian => 0.5,
            Difficulty::ExpertGuardian => 0.5,
            Difficulty::MasterGuardian => 0.5,
        }
    }

    /// Returns the sensitivity multiplier for the player's equipment.
    ///
    /// A higher value makes the equipment more responsive to paranormal activity.
    pub fn equipment_sensitivity(&self) -> f32 {
        match self {
            Difficulty::NoviceInvestigator => 1.3,
            Difficulty::AdeptInvestigator => 1.2,
            Difficulty::SeniorInvestigator => 1.1,
            Difficulty::ExpertInvestigator => 1.05,
            Difficulty::AdeptSpecialist => 1.0,
            Difficulty::LeadSpecialist => 0.95,
            Difficulty::ExpertSpecialist => 0.9,
            Difficulty::MasterSpecialist => 0.85,
            Difficulty::InitiateOccultist => 0.8,
            Difficulty::AdeptOccultist => 0.75,
            Difficulty::ExpertOccultist => 0.7,
            Difficulty::MasterOccultist => 0.65,
            Difficulty::AdeptGuardian => 0.6,
            Difficulty::LeadGuardian => 0.55,
            Difficulty::ExpertGuardian => 0.5,
            Difficulty::MasterGuardian => 0.4,
        }
    }

    // --- Gameplay ---
    /// Returns a boolean value indicating whether the van UI automatically opens at
    /// the beginning of a mission.
    pub fn van_auto_open(&self) -> bool {
        !matches!(
            self,
            Difficulty::NoviceInvestigator | Difficulty::AdeptInvestigator
        )
    }

    /// Returns the default tab selected in the van UI.
    pub fn default_van_tab(&self) -> TabContents {
        match self {
            Difficulty::NoviceInvestigator | Difficulty::AdeptInvestigator => TabContents::Journal,
            _ => TabContents::Loadout,
        }
    }

    pub fn player_gear(&self) -> PlayerGearKind {
        match self {
            Difficulty::NoviceInvestigator => PlayerGearKind {
                left_hand: GearKind::Flashlight,
                right_hand: GearKind::Thermometer,
                inventory: vec![GearKind::EMFMeter, GearKind::None],
            },
            Difficulty::AdeptInvestigator => PlayerGearKind {
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
            Difficulty::NoviceInvestigator => GhostSet::TmpEMF,
            Difficulty::AdeptInvestigator => GhostSet::TmpEMFUVOrbs,
            Difficulty::SeniorInvestigator => GhostSet::TmpEMFUVOrbsEVPCPM,
            Difficulty::ExpertInvestigator => GhostSet::Twenty,
            _ => GhostSet::All,
        }
    }

    pub fn truck_gear(&self) -> Vec<GearKind> {
        use crate::gear::ext::types::uncore_gearkind::GearKind::*;

        let mut gear = Vec::new();

        match self {
            Difficulty::NoviceInvestigator => {
                gear.push(Flashlight);
                gear.push(Thermometer);
                gear.push(EMFMeter);
            }
            Difficulty::AdeptInvestigator => {
                gear.extend(Self::NoviceInvestigator.truck_gear());
                gear.push(UVTorch);
                gear.push(Videocam);
            }
            Difficulty::SeniorInvestigator => {
                gear.extend(Self::AdeptInvestigator.truck_gear());
                gear.push(Recorder);
                gear.push(GeigerCounter);
            }
            Difficulty::ExpertInvestigator => {
                gear.extend(Self::SeniorInvestigator.truck_gear());
                gear.push(SpiritBox);
                gear.push(RedTorch);
            }
            Difficulty::AdeptSpecialist => {
                gear.extend(Self::ExpertInvestigator.truck_gear());
                gear.push(Salt);
                gear.push(QuartzStone);
                gear.push(SageBundle);
            }
            Difficulty::LeadSpecialist => {
                gear.extend(Self::AdeptSpecialist.truck_gear());
            }
            Difficulty::ExpertSpecialist => {
                gear.extend(Self::LeadSpecialist.truck_gear());
            }
            Difficulty::MasterSpecialist => {
                gear.extend(Self::ExpertSpecialist.truck_gear());
            }
            _ => {
                gear = Self::MasterSpecialist.truck_gear();
            }
        }

        // This is for debugging purposes, to add gear that isn't functional yet.
        const ENABLE_INCOMPLETE: bool = false;
        if ENABLE_INCOMPLETE {
            let mut incomplete: Vec<GearKind> = vec![
                // Incomplete equipment:
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
            Difficulty::NoviceInvestigator => "Novice Investigator",
            Difficulty::AdeptInvestigator => "Adept Investigator",
            Difficulty::SeniorInvestigator => "Senior Investigator",
            Difficulty::ExpertInvestigator => "Expert Investigator",
            Difficulty::AdeptSpecialist => "Adept Specialist",
            Difficulty::LeadSpecialist => "Lead Specialist",
            Difficulty::ExpertSpecialist => "Expert Specialist",
            Difficulty::MasterSpecialist => "Master Specialist",
            Difficulty::InitiateOccultist => "Initiate Occultist",
            Difficulty::AdeptOccultist => "Adept Occultist",
            Difficulty::ExpertOccultist => "Expert Occultist",
            Difficulty::MasterOccultist => "Master Occultist",
            Difficulty::AdeptGuardian => "Adept Guardian",
            Difficulty::LeadGuardian => "Lead Guardian",
            Difficulty::ExpertGuardian => "Expert Guardian",
            Difficulty::MasterGuardian => "Master Guardian",
        }
    }

    pub fn difficulty_description(&self) -> &'static str {
        match self {
            Difficulty::NoviceInvestigator => {
                "
                For those new to the paranormal.
                Friendly ghosts, minimal equipment, no risk of attacks. 
                Focus on mastering the basics with Thermometer and EMF Reader. (2/44 Ghosts)"
            },
            Difficulty::AdeptInvestigator => {
                "
                You've handled a few cases.
                Friendly ghosts, low risk, limited equipment.
                Add the UV Torch to your arsenal and learn to interpret its readings. (3/44 Ghosts)"
            },
            Difficulty::SeniorInvestigator => {
                "
                You're becoming familiar with the unseen.
                Access to standard equipment, low risk.
                Time to face a wider variety of ghosts and hone your investigative skills. (9/44 Ghosts)"
            },
            Difficulty::ExpertInvestigator => {
                "
                Your mind is strong, but the darkness lingers.
                Full equipment, low risk, slower sanity drain.
                Delve deeper into the mysteries and uncover the truth. (22/44 Ghosts)"
            },
            Difficulty::AdeptSpecialist => {
                "
                The veil thins, and the dangers increase.
                Average sanity drain, risk of ghostly attacks, haunted objects begin to manifest.
                Prepare for a true challenge. (All 44 Ghosts)"
            },
            Difficulty::LeadSpecialist => {
                "
                Your senses are tested, shadows play tricks.
                The unseen becomes harder to perceive, testing your observation skills and your courage.
                (All 44 Ghosts)"
            },
            Difficulty::ExpertSpecialist => {
                "
                The line blurs between the real and the spectral.
                Ghostly apparitions become more vivid, blurring the lines between sanity and madness.
                (All 44 Ghosts)"
            },
            Difficulty::MasterSpecialist => {
                "
                You walk a tightrope between worlds.
                The spirit realm intrudes upon reality, challenging your perception and your resolve. 
                (All 44 Ghosts)"
            },
            Difficulty::InitiateOccultist => {
                "
                The whispers of the ancients call to you.
                You arrive with a touch of madness, embracing the unknown. 
                Sanity drains faster, the unseen beckons. (All 44 Ghosts)"
            },
            Difficulty::AdeptOccultist => {
                "
                Ancient knowledge grants power, but at a cost.
                You start with a significant sanity deficit.
                Tread carefully, for the abyss gazes also into you. (All 44 Ghosts)"
            },
            Difficulty::ExpertOccultist => {
                "
                The whispers become screams, sanity teeters on the edge. 
                You begin deeply affected by the spirit world.  
                Only the most experienced should venture here. (All 44 Ghosts)"
            },
            Difficulty::MasterOccultist => {
                "
                Embrace the madness, for it holds the key. 
                Your sanity is a fragile thread, but your understanding of the paranormal is unmatched. 
                (All 44 Ghosts)"
            },
            Difficulty::AdeptGuardian => {
                "
                You stand as a shield against the darkness, but it takes its toll.
                Face relentless attacks, but your equipment is more attuned to the unseen. 
                (All 44 Ghosts)"
            },
            Difficulty::LeadGuardian => {
                "
                The spirits sense your strength and respond in kind.
                Prepare for intense confrontations, for your presence draws them out. 
                (All 44 Ghosts)"
            },
            Difficulty::ExpertGuardian => {
                "
                Your spirit shines brightly, a beacon in the night.
                The darkness seeks to extinguish your light, but your determination is unyielding.
                (All 44 Ghosts)"
            },
            Difficulty::MasterGuardian => {
                "
                You are a master of both worlds, walking the path between. 
                Face the ultimate challenges, for the fate of reality rests in your hands. 
                (All 44 Ghosts)"
            },
        }
    }

    /// Returns the score multiplier for the end-of-mission summary, based on the
    /// chosen difficulty level.
    ///
    /// Higher values reward players more for completing missions on harder
    /// difficulties.
    pub fn difficulty_score_multiplier(&self) -> f64 {
        match self {
            Difficulty::NoviceInvestigator => 1.0,
            Difficulty::AdeptInvestigator => 1.2,
            Difficulty::SeniorInvestigator => 1.5,
            Difficulty::ExpertInvestigator => 1.8,
            Difficulty::AdeptSpecialist => 2.1,
            Difficulty::LeadSpecialist => 2.4,
            Difficulty::ExpertSpecialist => 2.7,
            Difficulty::MasterSpecialist => 3.0,
            Difficulty::InitiateOccultist => 3.3,
            Difficulty::AdeptOccultist => 3.6,
            Difficulty::ExpertOccultist => 3.9,
            Difficulty::MasterOccultist => 4.2,
            Difficulty::AdeptGuardian => 4.5,
            Difficulty::LeadGuardian => 4.8,
            Difficulty::ExpertGuardian => 5.1,
            Difficulty::MasterGuardian => 6.0,
        }
    }

    pub fn tutorial_chapter(&self) -> Option<ManualChapter> {
        use crate::manual::{chapter1, chapter2, chapter3, chapter4, chapter5};
        match self {
            Difficulty::NoviceInvestigator => Some(chapter1::create_manual_chapter()),
            Difficulty::AdeptInvestigator => Some(chapter2::create_manual_chapter()),
            Difficulty::SeniorInvestigator => Some(chapter3::create_manual_chapter()),
            Difficulty::ExpertInvestigator => Some(chapter4::create_manual_chapter()),
            Difficulty::AdeptSpecialist => Some(chapter5::create_manual_chapter()),
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
            starting_sanity: self.starting_sanity(),
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
            difficulty_name: self.difficulty_name().to_owned(),
            difficulty_description: self.difficulty_description().to_owned(),
            difficulty_score_multiplier: self.difficulty_score_multiplier(),
            tutorial_chapter: self.tutorial_chapter(),
            truck_gear: self.truck_gear(),
        }
    }
}

/// Holds the concrete values for a specific difficulty level.
#[derive(Debug, Clone)]
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
    pub starting_sanity: f32,
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
    pub difficulty_description: String,
    pub difficulty_score_multiplier: f64,
    /// The range of manual pages associated with this difficulty.
    pub tutorial_chapter: Option<ManualChapter>,
    pub truck_gear: Vec<GearKind>,
}

#[derive(Debug, Resource, Clone)]
pub struct CurrentDifficulty(pub DifficultyStruct);

impl Default for CurrentDifficulty {
    fn default() -> Self {
        Self(Difficulty::default().create_difficulty_struct())
    }
}
