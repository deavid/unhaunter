//! Asset loading and Tiled map integration for Unhaunter
//!
//! This crate handles Bevy asset loading for game assets.

pub mod assets;
pub mod resources;
pub mod types;

pub use resources::cli_options::CliOptions;
pub use resources::maps::Maps;
pub use types::root::game_assets::GameAssets;
