use bevy::prelude::*;
use unbehavior_core::behavior::{Behavior, Util};
use unboard_core::components::spawning::PlayerSpawnPoint;
use unmapload_core::hydration::HydrationStage;

fn tag_player_spawn_points(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    for (entity, behavior) in q.iter_mut() {
        if let Util::PlayerSpawn = &behavior.p.util {
            commands.entity(entity).insert(PlayerSpawnPoint);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, tag_player_spawn_points);
}
