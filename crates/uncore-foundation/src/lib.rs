//! Foundation types for Unhaunter game
//!
//! This crate contains pure data types and enums with zero game logic.
//! Types are stable and widely used across the codebase.

pub mod colors;
pub mod platform;
pub mod random_seed;
pub mod types;
pub mod utils;

pub use colors::*;

// Temperature conversion utilities
pub const KELVIN_OFFSET: f32 = 273.15;

#[inline]
pub fn celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + KELVIN_OFFSET
}

#[inline]
pub fn kelvin_to_celsius(kelvin: f32) -> f32 {
    kelvin - KELVIN_OFFSET
}
