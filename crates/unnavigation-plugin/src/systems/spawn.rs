use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unnavigation_core::components::waypoint::WaypointQueue;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::resources::AuthorityRole;

/// Authority: inserts WaypointQueue on any player entity that is missing it.
/// Fires on the first Update frame after a PlayerSprite entity is spawned by the network layer.
fn hydrate_player_navigation(
    mut commands: Commands,
    q_new: Query<Entity, (With<PlayerSprite>, Without<WaypointQueue>)>,
) {
    for entity in q_new.iter() {
        commands.entity(entity).insert(WaypointQueue::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        hydrate_player_navigation
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
}
