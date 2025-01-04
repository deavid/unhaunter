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
pub mod activity;
pub mod craft_repellent;
pub mod journal;
pub mod journalui;
pub mod loadoutui;
pub mod plugin;
pub mod sanity;
pub mod sensors;
pub mod systems;
pub mod truckgear;
pub mod ui;
pub mod uibutton;
pub mod evidence;

use uncore::components::truck::{TruckUI, TruckUIGhostGuess};
use uncore::events::truck::TruckUIEvent;
use uncore::resources::ghost_guess::GhostGuess;
