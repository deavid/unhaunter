use bevy::prelude::*;
use unaudiospatial_core::listener::SpatialListener;
use unorchestrator_core::UIContextState;
use unplayer_core::components::PlayerTag;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unreplicon_core::resources::AuthorityRole;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;

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

/// Inserts `SpatialListener` when `MainPlayer` is added to an entity, and removes it
/// when `MainPlayer` is removed. Ensures at most one `SpatialListener` exists.
/// `MainPlayer` is the player domain's signal for "this is the local player" — it is
/// the right place to push the audio listener marker rather than having the audio
/// domain pull from player state.
fn sync_spatial_listener(
    mut commands: Commands,
    added: Query<Entity, Added<MainPlayer>>,
    mut removed: RemovedComponents<MainPlayer>,
) {
    for entity in removed.read() {
        commands.entity(entity).remove::<SpatialListener>();
    }
    for entity in added.iter() {
        commands.entity(entity).insert(SpatialListener);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            hydrate_player_spatial_tags.run_if(resource_exists::<AuthorityRole>),
            sync_spatial_listener,
        )
            .run_if(in_state(UIContextState::InGame)),
    );
}
