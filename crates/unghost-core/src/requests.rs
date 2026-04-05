use bevy::prelude::*;
use uninvestigation_core::ghost::GhostType;

/// Request component that signals ghost-logic (unghost-logic) to hydrate a skeleton ghost
/// entity into a fully configured ghost. Spawned by the orchestrator layer and consumed
/// (removed) by `ghost_hydration_system`.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct GhostSpawnRequest {
    pub ghost_types: Vec<GhostType>,
}
