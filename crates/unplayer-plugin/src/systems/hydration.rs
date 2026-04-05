use bevy::prelude::*;
use unaudiospatial_core::listener::SpatialListener;
use unbehavior_core::behavior::{Behavior, Util};
use unboard_core::components::spawning::PlayerSpawnPoint;
use uncommon_states_core::UIContextState;
use unmapload_core::hydration::HydrationStage;
use unmission_core::types::SimulationState;
use unplayer_core::components::{
    MainPlayer, PlayerDisconnected, PlayerSpawnRequest, PlayerSprite, PlayerTag,
};
use unreplicon_core::events::{PlayerNetworkDisconnected, PlayerNetworkReconnected};
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::resources::{AuthorityRole, LocalPlayer, LocalPlayerRole};
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;

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

/// Authority: materialize player-domain components from spawn request markers
/// emitted by networking systems.
fn materialize_players_from_spawn_requests(
    mut commands: Commands,
    q_spawn: Query<(Entity, &Position, &PlayerSpawnRequest), Without<PlayerSprite>>,
) {
    for (entity, pos, req) in q_spawn.iter() {
        info!(
            "materialize_players_from_spawn_requests: materializing entity {:?} for player {}",
            entity, req.player_uuid
        );
        commands.entity(entity).insert((
            unspatial_core::lerp_position::LerpPosition::new(*pos),
            PlayerSprite::new(req.player_uuid, req.network_id),
            unboard_core::resources::visibility_data::VisibilityData::default(),
        ));
        commands.entity(entity).remove::<PlayerSpawnRequest>();
    }
}

/// Authority: inserts PlayerTag and MapEntityFieldBPos on any player entity that is
/// missing them. MapEntityFieldBPos triggers board-field spatial sync.
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
/// when `MainPlayer` is removed.
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

/// Marks the UUID-matched locally owned player as the main local player.
fn insert_main_player_on_ownership(
    q: Query<(Entity, &PlayerSprite), (With<LocallyOwned>, Without<MainPlayer>)>,
    local_player: Res<LocalPlayer>,
    mut commands: Commands,
) {
    let Some(local_uuid) = local_player.0 else {
        return;
    };
    for (entity, sprite) in q.iter() {
        if sprite.id == local_uuid {
            commands.entity(entity).insert(MainPlayer);
        }
    }
}

/// Applies network lifecycle signals to player-domain disconnection markers.
fn apply_network_player_connection_state(
    mut disconnected_reader: MessageReader<PlayerNetworkDisconnected>,
    mut reconnected_reader: MessageReader<PlayerNetworkReconnected>,
    q_players: Query<(Entity, &PlayerSprite)>,
    mut commands: Commands,
) {
    for msg in disconnected_reader.read() {
        let mut found = false;
        for (entity, sprite) in q_players.iter() {
            if sprite.id == msg.player_uuid {
                commands.entity(entity).insert(PlayerDisconnected);
                found = true;
            }
        }
        if !found {
            warn!(
                "apply_network_player_connection_state: disconnect signal for unknown player {}",
                msg.player_uuid
            );
        }
    }

    for msg in reconnected_reader.read() {
        let mut found = false;
        for (entity, sprite) in q_players.iter() {
            if sprite.id == msg.player_uuid {
                commands.entity(entity).remove::<PlayerDisconnected>();
                found = true;
            }
        }
        if !found {
            warn!(
                "apply_network_player_connection_state: reconnect signal for unknown player {}",
                msg.player_uuid
            );
        }
    }
}

/// Authority-owned teardown for player entities.
fn despawn_players_on_teardown(
    mut commands: Commands,
    q_players: Query<Entity, With<PlayerSprite>>,
) {
    for entity in q_players.iter() {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, tag_player_spawn_points);
    app.add_systems(
        Update,
        (
            materialize_players_from_spawn_requests.run_if(resource_exists::<AuthorityRole>),
            insert_main_player_on_ownership
                .run_if(resource_exists::<LocalPlayerRole>)
                .after(materialize_players_from_spawn_requests),
        ),
    );
    app.add_systems(
        Update,
        (
            hydrate_player_spatial_tags.run_if(resource_exists::<AuthorityRole>),
            sync_spatial_listener,
            apply_network_player_connection_state.run_if(resource_exists::<AuthorityRole>),
        )
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        despawn_players_on_teardown.run_if(resource_exists::<AuthorityRole>),
    );
}
