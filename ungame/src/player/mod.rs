//! ## Player Module
//!
//! This module defines the player character and its interactions with the game.

pub mod animation;
pub mod components;
pub mod controlkeys;
pub mod setup;
pub mod systemparam;
pub mod systems;

pub use animation::{AnimationTimer, CharacterAnimation};
pub use components::deployedgear::{DeployedGear, DeployedGearData};
pub use components::playersprite::PlayerSprite;
pub use components::uncore_util::{HeldObject, Hiding};
pub use systemparam::interactivestuff::InteractiveStuff;

/// Enables the use of arrow keys for movement instead of WASD
const USE_ARROW_KEYS: bool = false;

/// Enables/disables debug logs related to the player.
const DEBUG_PLAYER: bool = false;

pub use setup::app_setup;
