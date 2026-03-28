use bevy::prelude::*;
use unbehavior_core::behavior::{Behavior, Util};
use unbehavior_core::components::InteractableByGhost;
use unboard_core::components::spawning::HostileSpawnPoint;
use unmapload_core::hydration::HydrationStage;
use unmetrics_core::metrics::SendMetric;

use crate::metrics;

fn hydration_ghost_logic_system(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    let measure = metrics::HYDRATION_GHOST_LOGIC.time_measure();
    for (entity, behavior) in q.iter_mut() {
        // Hostile (Ghost) Spawn Points
        if let Util::GhostSpawn = &behavior.p.util {
            commands.entity(entity).insert(HostileSpawnPoint);
        }

        // Add InteractableByGhost marker component for entities that ghosts can interact with
        let should_add_ghost_interaction =
            if behavior.p.is_door || behavior.p.is_switch || behavior.p.is_breaker {
                true
            } else {
                // For other classes, check properties
                let has_light = behavior.p.light.can_emit_light;
                let is_movable = behavior.p.object.movable;
                let is_throwable = behavior.p.object.throwable;
                let is_nudgeable = behavior.p.object.nudgeable;
                let haunt_movable = behavior.p.object.haunt_movable;

                has_light || is_movable || is_throwable || is_nudgeable || haunt_movable
            };

        if should_add_ghost_interaction {
            commands.entity(entity).insert(InteractableByGhost);
        }
    }
    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_ghost_logic_system);
}
