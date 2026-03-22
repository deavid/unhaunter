use bevy::prelude::*;
use unsettings_core::controls::ControlKeys;

/// Component that acts as a virtual joystick for player movement.
/// All input systems write to this component, and the movement system reads from it.
#[derive(Component, Debug, Clone)]
pub struct PlayerInput {
    /// The desired movement direction and magnitude.
    /// This is a normalized Vec2 where:
    /// - x represents left/right movement (-1 is left, 1 is right)
    /// - y represents up/down movement (-1 is down, 1 is up)
    ///
    /// If no movement is desired, this will be Vec2::ZERO
    pub movement: Vec2,

    /// Whether the player wants to run.
    pub run: bool,

    /// Whether the player wants to interact with an object.
    pub interact: bool,

    /// Whether the player wants to grab an object.
    pub grab: bool,

    /// Whether the player wants to drop an object.
    pub drop: bool,

    /// Whether the player wants to hide.
    pub hide_requested: bool,

    /// Whether the player wants to stop hiding.
    pub unhide_requested: bool,

    /// Whether the player wants to use the item in their right hand.
    pub use_right_hand: bool,

    /// Whether the player wants to use the item in their left hand.
    pub use_left_hand: bool,

    /// Cycle inventory.
    pub inventory_cycle: bool,

    /// Swap hands.
    pub inventory_swap: bool,

    /// The desired aiming direction and magnitude.
    pub aim_direction: Vec2,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            movement: Vec2::ZERO,
            run: false,
            interact: false,
            grab: false,
            drop: false,
            hide_requested: false,
            unhide_requested: false,
            use_right_hand: false,
            use_left_hand: false,
            inventory_cycle: false,
            inventory_swap: false,
            aim_direction: Vec2::ZERO,
        }
    }
}

impl PlayerInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear transient one-shot input events.
    ///
    /// Only clears events that should fire for exactly one frame (e.g., interact, grab).
    /// Persistent state like `movement`, `run`, and `aim_direction` is NOT cleared here
    /// because it represents ongoing input state that must be read by multiple systems
    /// within the same frame before being overwritten by the next frame's input.
    pub fn clear(&mut self) {
        self.interact = false;
        self.grab = false;
        self.drop = false;
        self.hide_requested = false;
        self.unhide_requested = false;
        self.use_right_hand = false;
        self.use_left_hand = false;
        self.inventory_cycle = false;
        self.inventory_swap = false;
    }
}

/// The keyboard control scheme for the player (WASD, IJKL, etc.).
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerInputMapping {
    pub controls: ControlKeys,
}
