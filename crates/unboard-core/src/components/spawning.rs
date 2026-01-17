use bevy::prelude::*;

/// A component that marks a location where a ghost can be spawned.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GhostSpawnPoint;

/// A component that marks a location where a player can be spawned.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerSpawnPoint;

/// A component that marks a location where a ghost breach/room-center can be spawned.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BreachSpawnPoint;

/// A component that marks a location where a van entry point is located.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct VanEntryPoint;
