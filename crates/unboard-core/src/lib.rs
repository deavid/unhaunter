//! Board and spatial systems for Unhaunter
//!
//! This crate contains the spatial positioning, collision, and board management systems.

use bevy::prelude::*;

pub mod components;
pub mod entity;
pub mod events;
pub mod resources;
pub mod types;
pub mod utils;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BoardUpdateSet {
    Collision,
    Lighting,
}
