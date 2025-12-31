//! Core game types for Unhaunter
//!
//! This crate contains game-specific types that bridge foundation types and components.

pub mod difficulty;
pub mod states;

pub use difficulty::Difficulty;
pub use states::*;
