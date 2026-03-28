use bevy::prelude::*;
use unplayer_core::components::PlayerSprite;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;
use untags_core::tags::PlayerTag;
use untypes_core::roles::AuthorityRole;
use untypes_core::states::AppState;

/// Authority: inserts PlayerTag and MapEntityFieldBPos on any player entity that is
/// missing them. Fires on the first Update frame after a PlayerSprite entity is spawned
/// by the network layer. MapEntityFieldBPos triggers the spatial-sync system used by
/// the board field — essential on dedicated servers where the render hydration is skipped.
fn hydrate_player_spatial_tags(
    mut commands: Commands,
    q_new: Query<(Entity, &Position), (With<PlayerSprite>, Without<PlayerTag>)>,
) {
    for (entity, pos) in q_new.iter() {
        commands
            .entity(entity)
            .insert((PlayerTag, MapEntityFieldBPos(pos.to_board_position())));
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        hydrate_player_spatial_tags
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(AppState::InGame)),
    );
}
