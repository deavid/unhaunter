use bevy::prelude::*;

/// Marker component for the camera used in menus.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MCamera;

/// Marker component for the UI root in menus.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuUI;
