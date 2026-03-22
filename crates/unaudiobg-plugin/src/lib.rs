//! Background audio track management plugin for Unhaunter
//!
//! Controls all continuous background audio (ambient, heartbeat, insanity, menu music).
//! Manages spawning, volume calculations, muting effects, and lifecycle.

pub mod plugin;
pub(crate) mod systems;
