use bevy::prelude::*;
use unbehavior::behavior::{Behavior, Util};
use unbehavior::components::InteractableByGhost;
use unboard_core::components::spawning::HostileSpawnPoint;
use untypes_core::hydration::HydrationStage;

fn hydration_ghost_logic_system(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    for (entity, behavior) in q.iter_mut() {
        let mut cmd = commands.entity(entity);

        // Hostile (Ghost) Spawn Points
        if let Util::GhostSpawn = &behavior.p.util {
            cmd.insert(HostileSpawnPoint);
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
            cmd.insert(InteractableByGhost);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_ghost_logic_system);
}
