use bevy::prelude::*;

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone)]
pub struct HeldObject {
    pub entity: Entity,
}

/// Marks a player entity that is currently hiding.
#[derive(Component)]
pub struct Hiding {
    pub hiding_spot: Entity,
}

/// Component for managing player stamina and running ability
#[derive(Component, Debug, Clone)]
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
