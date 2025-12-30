//! Marker components and shared data enums for decoupling game logic.

pub mod game;

use bevy::prelude::*;

/// Marker component for player entities.
#[derive(Component)]
pub struct PlayerTag {
    pub id: usize,
}

/// Marker component for ghost entities.
#[derive(Component)]
pub struct GhostTag;

/// Marker component for interactable entities.
#[derive(Component)]
pub struct InteractableTag;

/// Marker component for gear entities.
#[derive(Component)]
pub struct GearTag;

/// Marker component for NPC entities.
#[derive(Component)]
pub struct NpcTag;

/// Marker component for truck entities.
#[derive(Component)]
pub struct TruckTag;
