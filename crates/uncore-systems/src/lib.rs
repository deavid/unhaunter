//! Game systems, logic, and utilities for Unhaunter
//!
//! This crate contains all game logic, system implementations, and the core plugin.

pub mod noise;
pub mod platform;
pub mod plugin;
pub mod systemparam;
pub mod systems;
pub mod traits;
pub mod utils;

pub const DEBUG_PLAYER: bool = false;

// Re-export from uncore-foundation for backward compatibility
pub use uncore_foundation::{KELVIN_OFFSET, celsius_to_kelvin, kelvin_to_celsius};

pub use plugin::UnhaunterCorePlugin;
