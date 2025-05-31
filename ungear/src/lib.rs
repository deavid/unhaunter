//! ## Gear Module
//!
//! This module defines the gear system for the game, including:
//!
//! * Different types of gear available to the player.
//!
//! * A common interface for interacting with gear (`GearUsable` trait).
//!
//! * Functions for updating gear state based on player actions and game conditions.
//!
//! * Visual representations of gear using sprites and animations.
//!
//! The gear system allows players to equip and use various tools to investigate
//! paranormal activity, gather evidence, and ultimately banish ghosts.
pub mod components;
pub mod gear_stuff;
pub mod gear_usable;
pub mod plugin;
pub mod systems;
pub mod types;

pub use uncore::types::gear::spriteid::GearSpriteID;
