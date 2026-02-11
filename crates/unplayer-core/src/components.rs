use bevy::prelude::*;
use unfoundation_core::types::gear::Hand;
use unnet_core::messages::GearDetails;
use unnet_core::network_id::NetworkId;
use unsettings_core::controls::ControlKeys;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

#[derive(Component, Debug, Clone, Default)]
pub struct MainPlayer;

/// Component added to players who have disconnected but whose entity is being retained.
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerDisconnected;

/// Marks a player entity that is connected but unresponsive (no heartbeat for >5s).
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerInactive;

/// Component added to players who are spectating (dead or finished).
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerSpectating;

/// Component that acts as a virtual joystick for player movement.
/// All input systems write to this component, and the movement system reads from it.
#[derive(Component, Debug, Default, Clone)]
pub struct PlayerInput {
    /// The desired movement direction and magnitude.
    /// This is a normalized Vec2 where:
    /// - x represents left/right movement (-1 is left, 1 is right)
    /// - y represents up/down movement (-1 is down, 1 is up)
    ///
    /// If no movement is desired, this will be Vec2::ZERO
    pub movement: Vec2,

    /// Optional target position for click-to-move
    pub target_position: Option<Vec2>,

    /// Whether the player wants to run.
    pub run: bool,

    /// Whether the player wants to interact with an object.
    pub interact: bool,

    /// Whether the player wants to grab an object.
    pub grab: bool,

    /// Whether the player wants to drop an object.
    pub drop: bool,

    /// Whether the player wants to use the item in their right hand.
    pub use_right_hand: bool,

    /// Whether the player wants to use the item in their left hand.
    pub use_left_hand: bool,

    /// Predicted target state for the item in the right hand.
    pub target_right_hand: Option<(bool, GearDetails)>,

    /// Predicted target state for the item in the left hand.
    pub target_left_hand: Option<(bool, GearDetails)>,

    /// Cycle inventory.
    pub inventory_cycle: bool,

    /// Swap hands.
    pub inventory_swap: bool,

    /// The desired aiming direction and magnitude.
    pub aim_direction: Vec2,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear transient one-shot input events.
    ///
    /// Only clears events that should fire for exactly one frame (e.g., interact, grab).
    /// Persistent state like `movement`, `run`, and `aim_direction` is NOT cleared here
    /// because it must survive across frames for network sync — `client_send_input_system`
    /// in unnet-plugin may run before or after the input systems that set these fields.
    pub fn clear(&mut self) {
        self.interact = false;
        self.grab = false;
        self.drop = false;
        self.use_right_hand = false;
        self.use_left_hand = false;
        self.target_right_hand = None;
        self.target_left_hand = None;
        self.inventory_cycle = false;
        self.inventory_swap = false;
    }
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
/// This component stores the player's attributes, sanity level,
/// health, and mean sound exposure.
#[derive(Component, Debug)]
pub struct PlayerSprite {
    /// The unique identifier for the player (e.g., Player 1, Player 2).
    pub id: NetworkId,
    /// The player's accumulated "craziness" level. Higher craziness reduces sanity.
    pub crazyness: f32,
    /// The player's current sanity level (0.0 - 100.0).
    pub sanity: f32,
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

/// The keyboard control scheme for the player (WASD, IJKL, etc.).
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerInputMapping {
    pub controls: ControlKeys,
}

impl PlayerSprite {
    /// Creates a new `PlayerSprite` with the specified ID.
    pub fn new(id: NetworkId, spawn_position: Position) -> Self {
        Self {
            id,
            crazyness: 0.0,
            sanity: 100.0,
            mean_sound: 0.0,
            health: 100.0,
            spawn_position,
            movement: Direction::zero(),
        }
    }
}

/// Marks a player entity that is currently hiding.
#[derive(Component)]
pub struct Hiding {
    pub hiding_spot: Option<Entity>,
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
