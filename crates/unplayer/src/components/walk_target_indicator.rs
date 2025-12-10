use bevy::prelude::*;

/// Component that marks an entity as a walk target indicator.
/// This is spawned when the player has a MoveToTarget component
/// and despawned when the MoveToTarget component is removed.
#[derive(Component, Debug)]
pub struct WalkTargetIndicator;
