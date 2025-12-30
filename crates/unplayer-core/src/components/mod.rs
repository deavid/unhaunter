use bevy::prelude::*;
use uncore_foundation::types::gear::Hand;
use unsettings_core::controls::ControlKeys;
use unspatial_core::{Direction, Position};

/// Represents an object that is currently being held by the player.
#[derive(Component, Debug, Clone)]
pub struct HeldObject {
    pub entity: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct InventoryNext {
    pub idx: Option<usize>,
}

impl InventoryNext {
    pub fn new(idx: usize) -> Self {
        Self { idx: Some(idx) }
    }

    pub fn non_empty() -> Self {
        Self { idx: Some(0) }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub hand: Hand,
}

impl Inventory {
    pub fn new_left() -> Self {
        Inventory { hand: Hand::Left }
    }

    pub fn new_right() -> Self {
        Inventory { hand: Hand::Right }
    }
}

#[derive(Component, Debug, Clone)]
pub struct InventoryStats {
    pub hand: Hand,
}

impl InventoryStats {
    pub fn left() -> Self {
        InventoryStats { hand: Hand::Left }
    }
    pub fn right() -> Self {
        InventoryStats { hand: Hand::Right }
    }
}

/// Represents a player character in the game world.
///
/// This component stores the player's attributes, control scheme, sanity level,
/// health, and mean sound exposure.
#[derive(Component, Debug)]
pub struct PlayerSprite {
    /// The unique identifier for the player (e.g., Player 1, Player 2).
    pub id: usize,
    /// The keyboard control scheme for the player (WASD, IJKL, etc.).
    pub controls: ControlKeys,
    /// The player's accumulated "craziness" level. Higher craziness reduces sanity.
    pub crazyness: f32,
    /// The average sound level the player has been exposed to, used for sanity
    /// calculations.
    pub mean_sound: f32,
    /// The player's current health. A value of 0 indicates the player is incapacitated.
    pub health: f32,
    /// The player's initial spawn position when the level started.
    pub spawn_position: Position,
    /// The player's movement direction based on WASD controls.
    pub movement: Direction,
}

impl PlayerSprite {
    /// Creates a new `PlayerSprite` with the specified ID and default controls.
    pub fn new(id: usize, spawn_position: Position) -> Self {
        Self {
            id,
            controls: Self::default_controls(id),
            crazyness: 0.0,
            mean_sound: 0.0,
            health: 100.0,
            spawn_position,
            movement: Direction::zero(),
        }
    }

    /// Returns a modified version with the requested controls
    pub fn with_controls(self, controls: ControlKeys) -> Self {
        Self { controls, ..self }
    }

    /// Calculates the required crazyness based on the player's current sanity level.
    pub fn required_crazyness(sanity: f32) -> f32 {
        const LINEAR: f32 = 30.0;
        const SCALE: f32 = 100.0;
        (SCALE * LINEAR).powi(2) / (sanity * sanity) - LINEAR.powi(2)
    }

    /// Returns the default `ControlKeys` for the given player ID.
    fn default_controls(id: usize) -> ControlKeys {
        match id {
            1 => ControlKeys::WASD,
            2 => ControlKeys::IJKL,
            _ => ControlKeys::NONE,
        }
    }

    /// Calculates the player's current sanity level based on their accumulated
    /// craziness.
    pub fn sanity(&self) -> f32 {
        const LINEAR: f32 = 30.0;
        const SCALE: f32 = 100.0;
        (SCALE * LINEAR) / ((self.crazyness + LINEAR * LINEAR).max(0.01).sqrt())
    }
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
