use bevy::prelude::*;

/// Marker component for the camera used in the main game arena.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GCameraArena;

/// Marker component for the camera used in menus.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MCamera;
