//! ## Map Lighting and Visibility Module
//!
//! This module handles lighting, visibility, and color calculations for the game
//! world. It includes:
//!
//! * Functions for calculating the player's visibility field based on line-of-sight and potentially sanity levels.
//!
//! * Functions for applying lighting effects to map tiles and sprites, simulating various light sources (ambient, flashlight, ghost effects) and adjusting colors based on visibility and exposure.
//!
//! * Systems for dynamically updating lighting and visibility as the player moves and interacts with the environment.

pub(crate) mod definitions;
pub(crate) mod sampler;
pub(crate) mod systems;
pub(crate) mod visibility;
pub(crate) mod visuals;
