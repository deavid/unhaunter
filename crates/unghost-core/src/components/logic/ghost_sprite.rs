use bevy::prelude::*;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uncommon_app_core::random_seed;
use uninvestigation_core::evidence::Evidence;
use uninvestigation_core::ghost::GhostType;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

/// Per-ghost randomized noise offsets for unique behavior patterns
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct NoiseOffsets {
    // Evidence-specific offsets
    pub freezing_temp_x: f32,
    pub freezing_temp_y: f32,
    pub floating_orbs_x: f32,
    pub floating_orbs_y: f32,
    pub uv_ectoplasm_x: f32,
    pub uv_ectoplasm_y: f32,
    pub emf_level5_x: f32,
    pub emf_level5_y: f32,
    pub evp_recording_x: f32,
    pub evp_recording_y: f32,
    pub spirit_box_x: f32,
    pub spirit_box_y: f32,
    pub rl_presence_x: f32,
    pub rl_presence_y: f32,
    pub cpm500_x: f32,
    pub cpm500_y: f32,

    // Behavioral multiplier offsets
    pub visual_alpha_multiplier_x: f32,
    pub visual_alpha_multiplier_y: f32,
    pub rage_tendency_multiplier_x: f32,
    pub rage_tendency_multiplier_y: f32,
}

impl Default for NoiseOffsets {
    fn default() -> Self {
        Self::new_random()
    }
}

impl NoiseOffsets {
    /// Generate random noise offsets for a ghost
    pub fn new_random() -> Self {
        let mut rng = random_seed::rng();
        Self {
            freezing_temp_x: rng.random_range(0.0..100.0),
            freezing_temp_y: rng.random_range(0.0..100.0),
            floating_orbs_x: rng.random_range(0.0..100.0),
            floating_orbs_y: rng.random_range(0.0..100.0),
            uv_ectoplasm_x: rng.random_range(0.0..100.0),
            uv_ectoplasm_y: rng.random_range(0.0..100.0),
            emf_level5_x: rng.random_range(0.0..100.0),
            emf_level5_y: rng.random_range(0.0..100.0),
            evp_recording_x: rng.random_range(0.0..100.0),
            evp_recording_y: rng.random_range(0.0..100.0),
            spirit_box_x: rng.random_range(0.0..100.0),
            spirit_box_y: rng.random_range(0.0..100.0),
            rl_presence_x: rng.random_range(0.0..100.0),
            rl_presence_y: rng.random_range(0.0..100.0),
            cpm500_x: rng.random_range(0.0..100.0),
            cpm500_y: rng.random_range(0.0..100.0),
            visual_alpha_multiplier_x: rng.random_range(0.0..100.0),
            visual_alpha_multiplier_y: rng.random_range(0.0..100.0),
            rage_tendency_multiplier_x: rng.random_range(0.0..100.0),
            rage_tendency_multiplier_y: rng.random_range(0.0..100.0),
        }
    }

    /// Get noise offsets for a specific evidence type
    pub fn get_evidence_offsets(&self, evidence: Evidence) -> (f32, f32) {
        match evidence {
            Evidence::FreezingTemp => (self.freezing_temp_x, self.freezing_temp_y),
            Evidence::FloatingOrbs => (self.floating_orbs_x, self.floating_orbs_y),
            Evidence::UVEctoplasm => (self.uv_ectoplasm_x, self.uv_ectoplasm_y),
            Evidence::EMFLevel5 => (self.emf_level5_x, self.emf_level5_y),
            Evidence::EVPRecording => (self.evp_recording_x, self.evp_recording_y),
            Evidence::SpiritBox => (self.spirit_box_x, self.spirit_box_y),
            Evidence::RLPresence => (self.rl_presence_x, self.rl_presence_y),
            Evidence::CPM500 => (self.cpm500_x, self.cpm500_y),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct GhostBehaviorDynamics {
    pub freezing_temp_clarity: f32,
    pub floating_orbs_clarity: f32,
    pub uv_ectoplasm_clarity: f32,
    pub emf_level5_clarity: f32,
    pub evp_recording_clarity: f32,
    pub spirit_box_clarity: f32,
    pub rl_presence_clarity: f32,
    pub cpm500_clarity: f32,
    pub visual_alpha_multiplier: f32,
    pub rage_tendency_multiplier: f32,
    pub noise_offsets: NoiseOffsets,
}

impl Default for GhostBehaviorDynamics {
    fn default() -> Self {
        Self {
            freezing_temp_clarity: 1.0,
            floating_orbs_clarity: 1.0,
            uv_ectoplasm_clarity: 1.0,
            emf_level5_clarity: 1.0,
            evp_recording_clarity: 1.0,
            spirit_box_clarity: 1.0,
            rl_presence_clarity: 1.0,
            cpm500_clarity: 1.0,
            visual_alpha_multiplier: 1.0,
            rage_tendency_multiplier: 1.0,
            noise_offsets: NoiseOffsets::new_random(),
        }
    }
}

impl GhostBehaviorDynamics {
    pub fn get_clarity(&self, evidence: Evidence) -> f32 {
        match evidence {
            Evidence::FreezingTemp => self.freezing_temp_clarity,
            Evidence::FloatingOrbs => self.floating_orbs_clarity,
            Evidence::UVEctoplasm => self.uv_ectoplasm_clarity,
            Evidence::EMFLevel5 => self.emf_level5_clarity,
            Evidence::EVPRecording => self.evp_recording_clarity,
            Evidence::SpiritBox => self.spirit_box_clarity,
            Evidence::RLPresence => self.rl_presence_clarity,
            Evidence::CPM500 => self.cpm500_clarity,
        }
    }

    pub fn set_clarity(&mut self, evidence: Evidence, value: f32) {
        match evidence {
            Evidence::FreezingTemp => self.freezing_temp_clarity = value,
            Evidence::FloatingOrbs => self.floating_orbs_clarity = value,
            Evidence::UVEctoplasm => self.uv_ectoplasm_clarity = value,
            Evidence::EMFLevel5 => self.emf_level5_clarity = value,
            Evidence::EVPRecording => self.evp_recording_clarity = value,
            Evidence::SpiritBox => self.spirit_box_clarity = value,
            Evidence::RLPresence => self.rl_presence_clarity = value,
            Evidence::CPM500 => self.cpm500_clarity = value,
        }
    }
}

/// Represents a ghost entity in the game world.
///
/// Timer fields have been replaced with plain floats to avoid embedding Timer types
/// in a replicated component (Rule D: logic components must not contain Timers).
#[derive(Component, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct GhostSprite {
    /// The specific type of ghost, which determines its characteristics and abilities.
    pub class: GhostType,
    /// The ghost's designated spawn point (breach) on the game board.
    pub spawn_point: BoardPosition,
    /// The ghost's current target location in the game world.
    pub target_point: Option<Position>,
    /// Number of times the ghost has been hit with the correct type of repellent.
    pub repellent_hits: i64,
    /// Number of times the ghost has been hit with an incorrect type of repellent.
    pub repellent_misses: i64,
    /// Number of times the ghost has been hit with the correct type of repellent - in
    /// current frame.
    pub repellent_hits_frame: f32,
    /// Number of times the ghost has been hit with an incorrect type of repellent - in
    /// current frame.
    pub repellent_misses_frame: f32,
    /// Ghost is being hit with the correct type of repellent
    pub repellent_hits_delta: f32,
    /// Ghost is being hit with the incorrect type of repellent
    pub repellent_misses_delta: f32,
    /// The entity ID of the ghost's visual breach effect.
    pub breach_id: Option<Entity>,
    /// The ghost's current rage level, which influences its hunting behavior.
    pub rage: f32,
    /// Export of the rage limit for other systems to consider.
    pub rage_limit: f32,
    /// The ghost's hunting state. A value greater than 0 indicates active hunting.
    pub hunting: f32,
    /// Flag indicating whether the ghost is currently targeting a player during a hunt.
    pub hunt_target: bool,
    /// Time in seconds since the ghost started its current hunt.
    pub hunt_time_secs: f32,
    /// The ghost's current warping intensity, which affects its movement speed.
    pub warp: f32,
    /// The ghost got hit by sage, and it will be calm for a while.
    pub calm_time_secs: f32,
    /// Remaining seconds of the "Salty" side effect. 0.0 means inactive (effect finished).
    /// Set to a positive value (e.g. 120.0) when the ghost hits a salt pile.
    /// Replaces the old `salty_effect_timer: Timer`.
    pub salty_effect_remaining_secs: f32,
    /// Countdown in seconds until the next salty trace is spawned (repeating, 0.3s cycle).
    /// When this reaches 0 a trace is spawned and it resets to 0.3.
    /// Replaces the old `salty_trace_spawn_timer: Timer`.
    pub salty_trace_spawn_countdown_secs: f32,
    /// Makes the ghost wait more for the next attack but it will be a harder attack.
    pub rage_limit_multiplier: f32,
    /// Timer for pre-warning phase before hunt warning begins (anticipatory audio muting)
    pub pre_warning_timer: f32,
    /// True when in pre-hunt warning state
    pub hunt_warning_active: bool,
    /// Countdown timer (10 seconds)
    pub hunt_warning_timer: f32,
    /// Current warning wave intensity (0.0-1.0)
    pub hunt_warning_intensity: f32,
    /// Number of times the ghost has hunted in the current mission.
    pub times_hunted_this_mission: i64,
}

impl Default for GhostSprite {
    fn default() -> Self {
        Self {
            class: GhostType::default(),
            spawn_point: BoardPosition::default(),
            target_point: None,
            repellent_hits: 0,
            repellent_misses: 0,
            repellent_hits_frame: 0.0,
            repellent_misses_frame: 0.0,
            repellent_hits_delta: 0.0,
            repellent_misses_delta: 0.0,
            breach_id: None,
            rage: 0.0,
            hunting: 0.0,
            hunt_target: false,
            hunt_time_secs: 0.0,
            warp: 0.0,
            calm_time_secs: 0.0,
            salty_effect_remaining_secs: 0.0,
            salty_trace_spawn_countdown_secs: 0.3,
            rage_limit_multiplier: 1.0,
            rage_limit: 100.0,
            pre_warning_timer: 0.0,
            hunt_warning_active: false,
            hunt_warning_timer: 0.0,
            hunt_warning_intensity: 0.0,
            times_hunted_this_mission: 0,
        }
    }
}

impl GhostSprite {
    /// Creates a new `GhostSprite` with a random `GhostType` and the specified spawn point.
    pub fn new(spawn_point: BoardPosition, ghost_types: &[GhostType]) -> Self {
        let mut rng = random_seed::rng();
        let idx = rng.random_range(0..ghost_types.len());
        let class = ghost_types[idx];
        debug!("Ghost type: {:?} - {:?}", class, class.evidences());
        GhostSprite {
            class,
            spawn_point,
            ..Default::default()
        }
    }

    /// Sets the `breach_id` field, associating the ghost with its breach entity.
    pub fn with_breachid(self, breach_id: Entity) -> Self {
        Self {
            breach_id: Some(breach_id),
            ..self
        }
    }

    pub fn get_health(&self) -> f32 {
        1.0 - (self.repellent_hits as f32 / 1000.0)
    }

    /// Calculates a value from 0.0 to 1.0 representing how close the ghost is to hunting.
    pub fn hunt_likelihood(&self) -> f32 {
        10.0 / (self.rage_limit - self.rage).clamp(10.0, 10000.0)
    }
}
