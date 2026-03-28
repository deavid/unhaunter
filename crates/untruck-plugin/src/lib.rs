//! ## Truck UI Module
//!
//! This module defines the structure, layout, and behavior of the in-game truck
//! UI, which serves as the player's base of operations. It includes:
//!
//! * UI elements for managing player gear (loadout).
//!
//! * A journal for reviewing evidence and guessing the ghost type.
//!
//! * Displays for monitoring player sanity and sensor readings.
//!
//! * Buttons for crafting ghost repellents, exiting the truck, and ending the mission.
//!
//! The truck UI provides a centralized interface for players to interact with the
//! game's mechanics, track their progress, and make strategic decisions outside of
//! the main exploration and investigation gameplay.
pub(crate) mod hydration;
pub(crate) mod journal;
pub mod plugin;
pub(crate) mod systems;
pub(crate) mod truckgear;
pub(crate) mod types;
