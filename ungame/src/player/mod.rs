//! ## Player Module
//!
//! This module defines the player character and its interactions with the game.

pub mod setup;
pub mod systems;

pub use uncore::components::player_sprite::PlayerSprite;
pub use ungear::components::deployedgear::{DeployedGear, DeployedGearData};

pub use setup::app_setup;
