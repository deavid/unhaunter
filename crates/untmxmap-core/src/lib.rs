//! Tiled map and spritesheet loaders for Bevy.
//!
//! This crate provides Bevy asset loaders for Tiled Map Editor files (.tmx and .tsx),
//! including naive property parsing for efficient metadata extraction.

pub mod assets;
pub mod events;
pub mod resources;
pub mod tiled;
pub mod types;
