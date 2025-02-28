use std::time::Duration;

use bevy::prelude::*;
use rand::Rng as _;

use crate::{random_seed, types::ghost::types::GhostType};

use super::board::{boardposition::BoardPosition, position::Position};

/// Represents a ghost entity in the game world.
///
/// This component stores the ghost's type, spawn point, target location,
/// interaction stats, current mood, hunting state, and other relevant attributes.
#[derive(Component, Debug)]
pub struct GhostSprite {
    /// The specific type of ghost, which determines its characteristics and abilities.
    pub class: GhostType,
    /// The ghost's designated spawn point (breach) on the game board.
    pub spawn_point: BoardPosition,
    /// The ghost's current target location in the game world. `None` if the ghost is
    /// wandering aimlessly.
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
    /// The entity ID of the ghost's visual breach effect.
    pub breach_id: Option<Entity>,
    /// The ghost's current rage level, which influences its hunting behavior. Higher
    /// rage increases the likelihood of a hunt.
    pub rage: f32,
    /// The ghost's hunting state. A value greater than 0 indicates that the ghost is
    /// actively hunting a player.
    pub hunting: f32,
    /// Flag indicating whether the ghost is currently targeting a player during a hunt.
    pub hunt_target: bool,
    /// Time in seconds since the ghost started its current hunt.
    pub hunt_time_secs: f32,
    /// The ghost's current warping intensity, which affects its movement speed. Higher
    /// values result in faster warping.
    pub warp: f32,
    /// The ghost got hit by sage, and it will be calm for a while.
    pub calm_time_secs: f32,
    /// Timer to track the duration of the "Salty" side effect.
    pub salty_effect_timer: Timer,
    /// Timer to control the frequency of spawning Salty Traces.
    pub salty_trace_spawn_timer: Timer,
    /// Makes the ghost wait more for the next attack but it will be a harder attack.
    pub rage_limit_multiplier: f32,
}

impl GhostSprite {
    /// Creates a new `GhostSprite` with a random `GhostType` and the specified spawn
    /// point.
    ///
    /// The ghost's initial mood, hunting state, and other attributes are set to
    /// default values.
    pub fn new(spawn_point: BoardPosition, ghost_types: &[GhostType]) -> Self {
        let mut rng = random_seed::rng();
        let idx = rng.random_range(0..ghost_types.len());
        let class = ghost_types[idx];
        warn!("Ghost type: {:?} - {:?}", class, class.evidences());
        let mut salty_effect_timer = Timer::from_seconds(120.0, TimerMode::Once);
        salty_effect_timer.tick(Duration::from_secs(120));
        GhostSprite {
            class,
            spawn_point,
            target_point: None,
            repellent_hits: 0,
            repellent_misses: 0,
            repellent_hits_frame: 0.0,
            repellent_misses_frame: 0.0,
            breach_id: None,
            rage: 0.0,
            hunting: 0.0,
            hunt_target: false,
            hunt_time_secs: 0.0,
            warp: 0.0,
            calm_time_secs: 0.0,
            salty_effect_timer,
            salty_trace_spawn_timer: Timer::from_seconds(0.3, TimerMode::Repeating),
            rage_limit_multiplier: 1.0,
        }
    }

    /// Sets the `breach_id` field, associating the ghost with its visual breach effect.
    pub fn with_breachid(self, breach_id: Entity) -> Self {
        Self {
            breach_id: Some(breach_id),
            ..self
        }
    }
}
