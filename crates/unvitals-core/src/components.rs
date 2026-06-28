use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component for managing player vitals (health, sanity, etc.).
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerVitals {
    /// The player's accumulated "craziness" level. Higher craziness reduces sanity.
    pub crazyness: f32,
    /// The player's current sanity level (0.0 - 100.0).
    pub sanity: f32,
    /// The average sound level the player has been exposed to, used for sanity
    /// calculations.
    pub mean_sound: f32,
    /// The player's current health. A value of 0 indicates the player is incapacitated.
    pub health: f32,
    /// Immediate asphyxiation level (2-sec recovery). First-stage response to miasma.
    pub asphyxia_immediate: f32,
    /// Acute asphyxiation level (5-sec recovery). Responds to immediate exposure.
    pub asphyxia_acute: f32,
    /// Chronic asphyxiation level (60-sec recovery). Tracks accumulated miasma damage.
    pub asphyxia_chronic: f32,
}

impl MapEntities for PlayerVitals {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

impl Default for PlayerVitals {
    fn default() -> Self {
        Self {
            crazyness: 0.0,
            sanity: 100.0,
            mean_sound: 0.0,
            health: 100.0,
            asphyxia_immediate: 0.0,
            asphyxia_acute: 0.0,
            asphyxia_chronic: 0.0,
        }
    }
}

/// Component for managing player stamina and running ability
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct Stamina {
    /// Current stamina level
    pub current: f32,
    /// Maximum stamina level
    pub max: f32,
    /// Whether the player is currently running
    pub running: bool,
    /// Whether the player is exhausted (can't run until recovered)
    pub exhausted: bool,
    /// How quickly stamina depletes when running
    pub depletion_rate: f32,
    /// How quickly stamina recovers when not running
    pub recovery_rate: f32,
    /// Minimum stamina required to start running
    pub min_to_run: f32,
}

impl MapEntities for Stamina {
    fn map_entities<M: EntityMapper>(&mut self, _entity_mapper: &mut M) {}
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            running: false,
            exhausted: false,
            depletion_rate: 0.8, // Depletes by 0.8 per frame while running
            recovery_rate: 0.3,  // Recovers by 0.3 per frame when not running
            min_to_run: 30.0,    // Need at least 30% stamina to start running again
        }
    }
}

impl Stamina {
    pub fn is_able_to_run(&self) -> bool {
        !self.exhausted && self.current >= 0.0
    }

    pub fn update(&mut self, dt: f32, wants_to_run: bool) -> f32 {
        self.running = wants_to_run && self.is_able_to_run();

        if self.running {
            // Deplete stamina while running
            self.current -= self.depletion_rate * dt;
            if self.current <= 0.0 {
                self.current = 0.0;
                self.exhausted = true;
                self.running = false;
            }
            self.current / self.max
        } else {
            // Recover stamina when not running
            self.current += self.recovery_rate * dt;
            if self.current >= self.max {
                self.current = self.max;
                self.exhausted = false;
            } else if self.current >= self.min_to_run {
                // Once we recover enough stamina, we're no longer exhausted
                self.exhausted = false;
            }
            0.0
        }
    }

    /// Returns the current stamina percentage (0.0 - 1.0)
    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
}
