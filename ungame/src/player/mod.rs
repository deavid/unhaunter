//! ## Player Module
//!
//! This module defines the player character and its interactions with the game.

pub mod animation;
pub mod components;
pub mod setup;
pub mod systemparam;
pub mod systems;

pub use crate::gear::ext::components::deployedgear::{DeployedGear, DeployedGearData};
pub use animation::{AnimationTimer, CharacterAnimation};
pub use components::playersprite::PlayerSprite;
pub use components::uncore_util::{HeldObject, Hiding};
pub use systemparam::interactivestuff::InteractiveStuff;

pub use setup::app_setup;
