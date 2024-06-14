// src/difficulty.rs

//! Difficulty Module
//! -------------------
//!
//! This module defines the difficulty system for the Unhaunter game, enabling customization
//! of various gameplay parameters to provide a range of challenges for players.
//!
//! The `Difficulty` enum represents the different difficulty levels, each with a unique name
//! and associated settings.  The `DifficultyStruct` struct holds the concrete values
//! for a specific difficulty level, which are assembled using methods within the `Difficulty` enum.
//!
//! By defining these settings directly within the `Difficulty` enum, you can fine-tune
//! the game experience for each difficulty level, providing a tailored challenge for players.

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::truck::ui::TabContents;

/// Represents the different difficulty levels for the Unhaunter game.
///
/// Each variant corresponds to a specific difficulty preset with unique settings,
/// ranging from `Apprentice` (easiest) to `Legend` (hardest).
///
/// These difficulty levels impact various gameplay parameters, including ghost behavior,
/// environment conditions, player attributes, and general gameplay mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Serialize, Deserialize)]
pub enum Difficulty {
    Apprentice,
    FieldResearcher,
    ParanormalAnalyst,
    SeniorInvestigator,
    LeadResearcher,
    CaseManager,
    RegionalDirector,
    NationalSpecialist,
    GlobalExpert,
    Archivist,
    OccultScholar,
    Exorcist,
    Parapsychologist,
    SpiritualGuardian,
    UnhaunterMaster,
    Legend,
}

impl Difficulty {
    /// Returns an iterator over all difficulty levels.
    pub fn all() -> enum_iterator::All<Self> {
        all()
    }

    /// Returns the next difficulty level, wrapping around to the beginning if at the end.
    pub fn next(&self) -> Self {
        enum_iterator::next_cycle(self)
    }

    /// Returns the previous difficulty level, wrapping around to the end if at the beginning.
    pub fn prev(&self) -> Self {
        enum_iterator::previous_cycle(self)
    }

    // --- Ghost Behavior ---

    /// Returns the ghost's movement speed multiplier.
    ///
    /// A higher value indicates a faster ghost.
    pub fn ghost_speed(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.8,
            Difficulty::FieldResearcher => 0.9,
            Difficulty::ParanormalAnalyst => 1.0,
            Difficulty::SeniorInvestigator => 1.1,
            Difficulty::LeadResearcher => 1.2,
            Difficulty::CaseManager => 1.3,
            Difficulty::RegionalDirector => 1.4,
            Difficulty::NationalSpecialist => 1.5,
            Difficulty::GlobalExpert => 1.6,
            Difficulty::Archivist => 1.7,
            Difficulty::OccultScholar => 1.8,
            Difficulty::Exorcist => 1.9,
            Difficulty::Parapsychologist => 2.0,
            Difficulty::SpiritualGuardian => 2.1,
            Difficulty::UnhaunterMaster => 2.2,
            Difficulty::Legend => 2.5,
        }
    }

    /// Returns the ghost's rage buildup multiplier.
    ///
    /// A higher value means the ghost becomes enraged more quickly.
    pub fn ghost_rage_likelihood(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.6,
            Difficulty::FieldResearcher => 0.7,
            Difficulty::ParanormalAnalyst => 0.8,
            Difficulty::SeniorInvestigator => 0.9,
            Difficulty::LeadResearcher => 1.0,
            Difficulty::CaseManager => 1.1,
            Difficulty::RegionalDirector => 1.2,
            Difficulty::NationalSpecialist => 1.3,
            Difficulty::GlobalExpert => 1.4,
            Difficulty::Archivist => 1.5,
            Difficulty::OccultScholar => 1.6,
            Difficulty::Exorcist => 1.7,
            Difficulty::Parapsychologist => 1.8,
            Difficulty::SpiritualGuardian => 1.9,
            Difficulty::UnhaunterMaster => 2.0,
            Difficulty::Legend => 2.5,
        }
    }

    /// Returns the ghost's movement speed multiplier during a hunt.
    ///
    /// A higher value results in more aggressive pursuit of the player.
    pub fn ghost_hunting_aggression(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.9,
            Difficulty::FieldResearcher => 1.0,
            Difficulty::ParanormalAnalyst => 1.1,
            Difficulty::SeniorInvestigator => 1.2,
            Difficulty::LeadResearcher => 1.3,
            Difficulty::CaseManager => 1.4,
            Difficulty::RegionalDirector => 1.5,
            Difficulty::NationalSpecialist => 1.6,
            Difficulty::GlobalExpert => 1.7,
            Difficulty::Archivist => 1.8,
            Difficulty::OccultScholar => 1.9,
            Difficulty::Exorcist => 2.0,
            Difficulty::Parapsychologist => 2.1,
            Difficulty::SpiritualGuardian => 2.2,
            Difficulty::UnhaunterMaster => 2.3,
            Difficulty::Legend => 2.7,
        }
    }

    /// Returns the multiplier for how often the ghost interacts with objects or triggers events.
    ///
    /// A higher value leads to more frequent paranormal activity.
    pub fn ghost_interaction_frequency(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.5,
            Difficulty::FieldResearcher => 0.7,
            Difficulty::ParanormalAnalyst => 0.9,
            Difficulty::SeniorInvestigator => 1.1,
            Difficulty::LeadResearcher => 1.3,
            Difficulty::CaseManager => 1.5,
            Difficulty::RegionalDirector => 1.7,
            Difficulty::NationalSpecialist => 1.9,
            Difficulty::GlobalExpert => 2.1,
            Difficulty::Archivist => 2.3,
            Difficulty::OccultScholar => 2.5,
            Difficulty::Exorcist => 2.7,
            Difficulty::Parapsychologist => 2.9,
            Difficulty::SpiritualGuardian => 3.1,
            Difficulty::UnhaunterMaster => 3.3,
            Difficulty::Legend => 4.0,
        }
    }

    /// Returns the multiplier for the duration of a ghost's hunting phase.
    ///
    /// A higher value results in longer hunts.
    pub fn ghost_hunt_duration(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.7,
            Difficulty::FieldResearcher => 0.8,
            Difficulty::ParanormalAnalyst => 0.9,
            Difficulty::SeniorInvestigator => 1.0,
            Difficulty::LeadResearcher => 1.1,
            Difficulty::CaseManager => 1.2,
            Difficulty::RegionalDirector => 1.3,
            Difficulty::NationalSpecialist => 1.4,
            Difficulty::GlobalExpert => 1.5,
            Difficulty::Archivist => 1.6,
            Difficulty::OccultScholar => 1.7,
            Difficulty::Exorcist => 1.8,
            Difficulty::Parapsychologist => 1.9,
            Difficulty::SpiritualGuardian => 2.0,
            Difficulty::UnhaunterMaster => 2.1,
            Difficulty::Legend => 2.5,
        }
    }

    /// Returns the multiplier for the time between ghost hunts.
    ///
    /// A higher value means longer periods of calm between hunts.
    pub fn ghost_hunt_cooldown(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.5,
            Difficulty::FieldResearcher => 1.3,
            Difficulty::ParanormalAnalyst => 1.1,
            Difficulty::SeniorInvestigator => 1.0,
            Difficulty::LeadResearcher => 0.9,
            Difficulty::CaseManager => 0.8,
            Difficulty::RegionalDirector => 0.7,
            Difficulty::NationalSpecialist => 0.65,
            Difficulty::GlobalExpert => 0.6,
            Difficulty::Archivist => 0.55,
            Difficulty::OccultScholar => 0.5,
            Difficulty::Exorcist => 0.45,
            Difficulty::Parapsychologist => 0.4,
            Difficulty::SpiritualGuardian => 0.35,
            Difficulty::UnhaunterMaster => 0.3,
            Difficulty::Legend => 0.2,
        }
    }

    /// Returns the ghost's attraction factor to its breach (spawn point).
    ///
    /// A higher value means the ghost tends to stay closer to its breach,
    /// while a lower value allows the ghost to roam more freely.
    pub fn ghost_attraction_to_breach(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.8,
            Difficulty::FieldResearcher => 1.6,
            Difficulty::ParanormalAnalyst => 1.4,
            Difficulty::SeniorInvestigator => 1.2,
            Difficulty::LeadResearcher => 1.0,
            Difficulty::CaseManager => 0.9,
            Difficulty::RegionalDirector => 0.8,
            Difficulty::NationalSpecialist => 0.7,
            Difficulty::GlobalExpert => 0.6,
            Difficulty::Archivist => 0.55,
            Difficulty::OccultScholar => 0.5,
            Difficulty::Exorcist => 0.45,
            Difficulty::Parapsychologist => 0.4,
            Difficulty::SpiritualGuardian => 0.35,
            Difficulty::UnhaunterMaster => 0.3,
            Difficulty::Legend => 0.2,
        }
    }

    /// Returns the radius around the ghost's breach within which a Repulsive object
    /// can provoke a hunt.
    pub fn hunt_provocation_radius(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.0,
            Difficulty::FieldResearcher => 1.5,
            Difficulty::ParanormalAnalyst => 2.0,
            Difficulty::SeniorInvestigator => 2.5,
            Difficulty::LeadResearcher => 3.0,
            Difficulty::CaseManager => 3.2,
            Difficulty::RegionalDirector => 3.4,
            Difficulty::NationalSpecialist => 3.6,
            Difficulty::GlobalExpert => 3.8,
            Difficulty::Archivist => 4.0,
            Difficulty::OccultScholar => 4.2,
            Difficulty::Exorcist => 4.4,
            Difficulty::Parapsychologist => 4.6,
            Difficulty::SpiritualGuardian => 4.8,
            Difficulty::UnhaunterMaster => 5.0,
            Difficulty::Legend => 5.5,
        }
    }

    /// Returns the rate at which the ghost's anger increases when an attractive
    /// object is removed from the location.
    pub fn attractive_removal_anger_rate(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.02,
            Difficulty::FieldResearcher => 0.03,
            Difficulty::ParanormalAnalyst => 0.04,
            Difficulty::SeniorInvestigator => 0.05,
            Difficulty::LeadResearcher => 0.06,
            Difficulty::CaseManager => 0.07,
            Difficulty::RegionalDirector => 0.08,
            Difficulty::NationalSpecialist => 0.09,
            Difficulty::GlobalExpert => 0.1,
            Difficulty::Archivist => 0.11,
            Difficulty::OccultScholar => 0.12,
            Difficulty::Exorcist => 0.13,
            Difficulty::Parapsychologist => 0.14,
            Difficulty::SpiritualGuardian => 0.15,
            Difficulty::UnhaunterMaster => 0.16,
            Difficulty::Legend => 0.2,
        }
    }

    // --- Environment ---

    /// Returns the base ambient temperature of the location.
    pub fn ambient_temperature(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 20.0,
            Difficulty::FieldResearcher => 18.0,
            Difficulty::ParanormalAnalyst => 16.0,
            Difficulty::SeniorInvestigator => 14.0,
            Difficulty::LeadResearcher => 12.0,
            Difficulty::CaseManager => 10.0,
            Difficulty::RegionalDirector => 8.0,
            Difficulty::NationalSpecialist => 7.0,
            Difficulty::GlobalExpert => 6.0,
            Difficulty::Archivist => 5.5,
            Difficulty::OccultScholar => 5.0,
            Difficulty::Exorcist => 4.5,
            Difficulty::Parapsychologist => 4.0,
            Difficulty::SpiritualGuardian => 3.5,
            Difficulty::UnhaunterMaster => 3.0,
            Difficulty::Legend => 2.0,
        }
    }

    /// Returns the multiplier for how quickly temperature changes
    /// propagate through the environment.
    pub fn temperature_spread_speed(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.7,
            Difficulty::FieldResearcher => 0.8,
            Difficulty::ParanormalAnalyst => 0.9,
            Difficulty::SeniorInvestigator => 1.0,
            Difficulty::LeadResearcher => 1.1,
            Difficulty::CaseManager => 1.2,
            Difficulty::RegionalDirector => 1.25,
            Difficulty::NationalSpecialist => 1.3,
            Difficulty::GlobalExpert => 1.35,
            Difficulty::Archivist => 1.4,
            Difficulty::OccultScholar => 1.45,
            Difficulty::Exorcist => 1.5,
            Difficulty::Parapsychologist => 1.55,
            Difficulty::SpiritualGuardian => 1.6,
            Difficulty::UnhaunterMaster => 1.65,
            Difficulty::Legend => 1.7,
        }
    }

    /// Returns the heat generation multiplier for light sources.
    ///
    /// A higher value means lights emit more heat.
    pub fn light_heat(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.8,
            Difficulty::FieldResearcher => 0.9,
            Difficulty::ParanormalAnalyst => 1.0,
            Difficulty::SeniorInvestigator => 1.05,
            Difficulty::LeadResearcher => 1.1,
            Difficulty::CaseManager => 1.15,
            Difficulty::RegionalDirector => 1.2,
            Difficulty::NationalSpecialist => 1.25,
            Difficulty::GlobalExpert => 1.3,
            Difficulty::Archivist => 1.35,
            Difficulty::OccultScholar => 1.4,
            Difficulty::Exorcist => 1.45,
            Difficulty::Parapsychologist => 1.5,
            Difficulty::SpiritualGuardian => 1.55,
            Difficulty::UnhaunterMaster => 1.6,
            Difficulty::Legend => 1.7,
        }
    }

    /// Returns the multiplier for the intensity of darkness at low light levels.
    ///
    /// This simulates the eye's adaptation to darkness, with higher values resulting
    /// in a darker environment overall.
    pub fn darkness_intensity(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.6,
            Difficulty::FieldResearcher => 0.7,
            Difficulty::ParanormalAnalyst => 0.8,
            Difficulty::SeniorInvestigator => 0.9,
            Difficulty::LeadResearcher => 1.0,
            Difficulty::CaseManager => 1.05,
            Difficulty::RegionalDirector => 1.1,
            Difficulty::NationalSpecialist => 1.15,
            Difficulty::GlobalExpert => 1.2,
            Difficulty::Archivist => 1.25,
            Difficulty::OccultScholar => 1.3,
            Difficulty::Exorcist => 1.35,
            Difficulty::Parapsychologist => 1.4,
            Difficulty::SpiritualGuardian => 1.45,
            Difficulty::UnhaunterMaster => 1.5,
            Difficulty::Legend => 1.6,
        }
    }

    /// Returns the gamma correction value for the game environment.
    ///
    /// Higher values make the lighting appear harsher and less realistic,
    /// potentially highlighting details but creating a less atmospheric look.
    pub fn environment_gamma(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.8,
            Difficulty::FieldResearcher => 1.6,
            Difficulty::ParanormalAnalyst => 1.4,
            Difficulty::SeniorInvestigator => 1.3,
            Difficulty::LeadResearcher => 1.2,
            Difficulty::CaseManager => 1.15,
            Difficulty::RegionalDirector => 1.1,
            Difficulty::NationalSpecialist => 1.05,
            Difficulty::GlobalExpert => 1.0,
            Difficulty::Archivist => 0.95,
            Difficulty::OccultScholar => 0.9,
            Difficulty::Exorcist => 0.85,
            Difficulty::Parapsychologist => 0.8,
            Difficulty::SpiritualGuardian => 0.75,
            Difficulty::UnhaunterMaster => 0.7,
            Difficulty::Legend => 0.6,
        }
    }

    // --- Player ---

    /// Returns the player's starting sanity level.
    pub fn starting_sanity(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 100.0,
            Difficulty::FieldResearcher => 90.0,
            Difficulty::ParanormalAnalyst => 80.0,
            Difficulty::SeniorInvestigator => 75.0,
            Difficulty::LeadResearcher => 70.0,
            Difficulty::CaseManager => 65.0,
            Difficulty::RegionalDirector => 60.0,
            Difficulty::NationalSpecialist => 55.0,
            Difficulty::GlobalExpert => 50.0,
            Difficulty::Archivist => 45.0,
            Difficulty::OccultScholar => 40.0,
            Difficulty::Exorcist => 35.0,
            Difficulty::Parapsychologist => 30.0,
            Difficulty::SpiritualGuardian => 25.0,
            Difficulty::UnhaunterMaster => 20.0,
            Difficulty::Legend => 10.0,
        }
    }

    /// Returns the sanity drain rate multiplier.
    ///
    /// A higher value results in faster sanity loss for the player.
    pub fn sanity_drain_rate(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.5,
            Difficulty::FieldResearcher => 0.7,
            Difficulty::ParanormalAnalyst => 0.9,
            Difficulty::SeniorInvestigator => 1.1,
            Difficulty::LeadResearcher => 1.3,
            Difficulty::CaseManager => 1.5,
            Difficulty::RegionalDirector => 1.7,
            Difficulty::NationalSpecialist => 1.9,
            Difficulty::GlobalExpert => 2.1,
            Difficulty::Archivist => 2.3,
            Difficulty::OccultScholar => 2.5,
            Difficulty::Exorcist => 2.7,
            Difficulty::Parapsychologist => 2.9,
            Difficulty::SpiritualGuardian => 3.1,
            Difficulty::UnhaunterMaster => 3.3,
            Difficulty::Legend => 4.0,
        }
    }

    /// Returns the health drain rate multiplier during a ghost hunt.
    ///
    /// A higher value means the player loses health more quickly during hunts.
    pub fn health_drain_rate(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 0.8,
            Difficulty::FieldResearcher => 0.9,
            Difficulty::ParanormalAnalyst => 1.0,
            Difficulty::SeniorInvestigator => 1.1,
            Difficulty::LeadResearcher => 1.2,
            Difficulty::CaseManager => 1.3,
            Difficulty::RegionalDirector => 1.4,
            Difficulty::NationalSpecialist => 1.5,
            Difficulty::GlobalExpert => 1.6,
            Difficulty::Archivist => 1.7,
            Difficulty::OccultScholar => 1.8,
            Difficulty::Exorcist => 1.9,
            Difficulty::Parapsychologist => 2.0,
            Difficulty::SpiritualGuardian => 2.1,
            Difficulty::UnhaunterMaster => 2.2,
            Difficulty::Legend => 2.5,
        }
    }

    /// Returns the health recovery rate multiplier.
    ///
    /// A higher value means the player regains health more quickly.
    pub fn health_recovery_rate(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.2,
            Difficulty::FieldResearcher => 1.1,
            Difficulty::ParanormalAnalyst => 1.0,
            Difficulty::SeniorInvestigator => 0.95,
            Difficulty::LeadResearcher => 0.9,
            Difficulty::CaseManager => 0.85,
            Difficulty::RegionalDirector => 0.8,
            Difficulty::NationalSpecialist => 0.75,
            Difficulty::GlobalExpert => 0.7,
            Difficulty::Archivist => 0.65,
            Difficulty::OccultScholar => 0.6,
            Difficulty::Exorcist => 0.55,
            Difficulty::Parapsychologist => 0.5,
            Difficulty::SpiritualGuardian => 0.45,
            Difficulty::UnhaunterMaster => 0.4,
            Difficulty::Legend => 0.3,
        }
    }

    /// Returns the player's movement speed multiplier.
    pub fn player_speed(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.1,
            Difficulty::FieldResearcher => 1.0,
            Difficulty::ParanormalAnalyst => 0.95,
            Difficulty::SeniorInvestigator => 0.9,
            Difficulty::LeadResearcher => 0.85,
            Difficulty::CaseManager => 0.8,
            Difficulty::RegionalDirector => 0.75,
            Difficulty::NationalSpecialist => 0.7,
            Difficulty::GlobalExpert => 0.65,
            Difficulty::Archivist => 0.6,
            Difficulty::OccultScholar => 0.55,
            Difficulty::Exorcist => 0.5,
            Difficulty::Parapsychologist => 0.45,
            Difficulty::SpiritualGuardian => 0.4,
            Difficulty::UnhaunterMaster => 0.35,
            Difficulty::Legend => 0.3,
        }
    }

    // --- Evidence Gathering ---

    /// Returns the multiplier for the clarity or intensity of evidence manifestations.
    ///
    /// A higher value makes evidence more visible or noticeable.
    pub fn evidence_visibility(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.5,
            Difficulty::FieldResearcher => 1.3,
            Difficulty::ParanormalAnalyst => 1.1,
            Difficulty::SeniorInvestigator => 1.0,
            Difficulty::LeadResearcher => 0.9,
            Difficulty::CaseManager => 0.8,
            Difficulty::RegionalDirector => 0.7,
            Difficulty::NationalSpecialist => 0.65,
            Difficulty::GlobalExpert => 0.6,
            Difficulty::Archivist => 0.55,
            Difficulty::OccultScholar => 0.5,
            Difficulty::Exorcist => 0.45,
            Difficulty::Parapsychologist => 0.4,
            Difficulty::SpiritualGuardian => 0.35,
            Difficulty::UnhaunterMaster => 0.3,
            Difficulty::Legend => 0.2,
        }
    }

    /// Returns the sensitivity multiplier for the player's equipment.
    ///
    /// A higher value makes the equipment more responsive to paranormal activity.
    pub fn equipment_sensitivity(&self) -> f32 {
        match self {
            Difficulty::Apprentice => 1.3,
            Difficulty::FieldResearcher => 1.2,
            Difficulty::ParanormalAnalyst => 1.1,
            Difficulty::SeniorInvestigator => 1.05,
            Difficulty::LeadResearcher => 1.0,
            Difficulty::CaseManager => 0.95,
            Difficulty::RegionalDirector => 0.9,
            Difficulty::NationalSpecialist => 0.85,
            Difficulty::GlobalExpert => 0.8,
            Difficulty::Archivist => 0.75,
            Difficulty::OccultScholar => 0.7,
            Difficulty::Exorcist => 0.65,
            Difficulty::Parapsychologist => 0.6,
            Difficulty::SpiritualGuardian => 0.55,
            Difficulty::UnhaunterMaster => 0.5,
            Difficulty::Legend => 0.4,
        }
    }

    // --- Gameplay ---

    /// Returns a boolean value indicating whether the van UI automatically opens
    /// at the beginning of a mission.
    pub fn van_auto_open(&self) -> bool {
        matches!(
            self,
            Difficulty::Apprentice
                | Difficulty::FieldResearcher
                | Difficulty::ParanormalAnalyst
                | Difficulty::SeniorInvestigator
                | Difficulty::LeadResearcher
                | Difficulty::CaseManager
        )
    }

    /// Returns the default tab selected in the van UI.
    pub fn default_van_tab(&self) -> TabContents {
        match self {
            Difficulty::Apprentice | Difficulty::FieldResearcher => TabContents::Loadout,
            _ => TabContents::Journal,
        }
    }

    /// Returns a boolean value indicating whether players are allowed to discard evidence
    /// in the journal.
    pub fn allow_evidence_discard(&self) -> bool {
        matches!(
            self,
            Difficulty::Apprentice
                | Difficulty::FieldResearcher
                | Difficulty::ParanormalAnalyst
                | Difficulty::SeniorInvestigator
        )
    }

    /// Returns the maximum number of inventory slots the player has available.
    pub fn max_inventory_slots(&self) -> usize {
        match self {
            Difficulty::Apprentice => 4,
            Difficulty::FieldResearcher
            | Difficulty::ParanormalAnalyst
            | Difficulty::SeniorInvestigator => 3,
            _ => 2,
        }
    }

    // --- UI and Scoring ---

    /// Returns the display name for the difficulty level.
    pub fn difficulty_name(&self) -> &'static str {
        match self {
            Difficulty::Apprentice => "Apprentice",
            Difficulty::FieldResearcher => "Field Researcher",
            Difficulty::ParanormalAnalyst => "Paranormal Analyst",
            Difficulty::SeniorInvestigator => "Senior Investigator",
            Difficulty::LeadResearcher => "Lead Researcher",
            Difficulty::CaseManager => "Case Manager",
            Difficulty::RegionalDirector => "Regional Director",
            Difficulty::NationalSpecialist => "National Specialist",
            Difficulty::GlobalExpert => "Global Expert",
            Difficulty::Archivist => "Archivist",
            Difficulty::OccultScholar => "Occult Scholar",
            Difficulty::Exorcist => "Exorcist",
            Difficulty::Parapsychologist => "Parapsychologist",
            Difficulty::SpiritualGuardian => "Spiritual Guardian",
            Difficulty::UnhaunterMaster => "Unhaunter Master",
            Difficulty::Legend => "Legend",
        }
    }

    /// Returns the score multiplier for the end-of-mission summary,
    /// based on the chosen difficulty level.
    ///
    /// Higher values reward players more for completing missions on harder difficulties.
    pub fn difficulty_score_multiplier(&self) -> f64 {
        match self {
            Difficulty::Apprentice => 1.0,
            Difficulty::FieldResearcher => 1.2,
            Difficulty::ParanormalAnalyst => 1.5,
            Difficulty::SeniorInvestigator => 1.8,
            Difficulty::LeadResearcher => 2.1,
            Difficulty::CaseManager => 2.4,
            Difficulty::RegionalDirector => 2.7,
            Difficulty::NationalSpecialist => 3.0,
            Difficulty::GlobalExpert => 3.3,
            Difficulty::Archivist => 3.6,
            Difficulty::OccultScholar => 3.9,
            Difficulty::Exorcist => 4.2,
            Difficulty::Parapsychologist => 4.5,
            Difficulty::SpiritualGuardian => 4.8,
            Difficulty::UnhaunterMaster => 5.1,
            Difficulty::Legend => 6.0,
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
            allow_evidence_discard: self.allow_evidence_discard(),
            max_inventory_slots: self.max_inventory_slots(),
            difficulty_name: self.difficulty_name().to_owned(), // Clone the name
            difficulty_score_multiplier: self.difficulty_score_multiplier(),
        }
    }
}

/// Holds the concrete values for a specific difficulty level.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DifficultyStruct {
    // --- Ghost Behavior ---
    pub ghost_speed: f32,
    pub ghost_rage_likelihood: f32,
    pub ghost_hunting_aggression: f32,
    pub ghost_interaction_frequency: f32,
    pub ghost_hunt_duration: f32,
    pub ghost_hunt_cooldown: f32,
    pub ghost_attraction_to_breach: f32,
    pub hunt_provocation_radius: f32,
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
    pub allow_evidence_discard: bool,
    pub max_inventory_slots: usize,

    // --- UI and Scoring ---
    pub difficulty_name: String,
    pub difficulty_score_multiplier: f64,
}
