//! Classic mode rendering systems for Unhaunter.
//!
//! This plugin implements the graphics and visualization layer for Classic Mode,
//! including entity hydration (mesh/materials setup), camera control, and visual synchronization.

pub(crate) mod camera;
pub(crate) mod cleanup;
pub(crate) mod hydration;

pub mod plugin;
