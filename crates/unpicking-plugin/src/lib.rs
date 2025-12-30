//! Custom picking backend for map sprites
//!
//! This module provides a custom picking backend for map sprites that use custom
//! materials instead of the standard `Sprite` component. It enables mouse interaction
//! with doors, switches, and other interactive map elements.

mod plugin;
mod sprite_picking_backend;

pub use plugin::CustomSpritePickingPlugin;
