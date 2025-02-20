use crate::controlkeys::ControlKeys;
use bevy::prelude::*;

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
}

impl PlayerSprite {
    /// Creates a new `PlayerSprite` with the specified ID and default controls.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            controls: Self::default_controls(id),
            crazyness: 0.0,
            mean_sound: 0.0,
            health: 100.0,
        }
    }

    /// Returns a modified version with the requested sanity
    pub fn with_sanity(self, sanity: f32) -> Self {
        Self {
            crazyness: Self::required_crazyness(sanity),
            ..self
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
        (SCALE * LINEAR) / ((self.crazyness + LINEAR * LINEAR).sqrt())
    }
}
