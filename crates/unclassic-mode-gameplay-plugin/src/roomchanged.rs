use bevy::prelude::*;
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unclassic_mode_core::components::GCameraArena;
use uninteraction_core::events::RoomChangedEvent;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use untruck_core::components::in_truck::InTruck;

/// Handles `RoomChangedEvent` events, updating interactive object states and room
/// data.
///
/// This system is responsible for:
///
/// * Updating the state of interactive objects based on the current room's state.
///
/// * Triggering the opening of the van UI when appropriate (e.g., when the player
///   enters the starting area).
///
/// * Updating the game's collision and lighting data after room-related changes.
fn roomchanged_event(
    mut commands: Commands,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
    mut ev_room: MessageReader<RoomChangedEvent>,
    pc: Query<(Entity, &PlayerSprite, &Transform), (Without<GCameraArena>, With<MainPlayer>)>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
) {
    let mut any_initialized = false;
    let mut any_open_van = false;
    let mut any_event = false;

    for ev in ev_room.read() {
        any_event = true;
        if ev.initialize {
            any_initialized = true;
        }
        if ev.open_van {
            any_open_van = true;
        }
    }

    if !any_event {
        return;
    }

    ev_bdr.write(BoardTopologyToRebuild {
        lighting: true,
        collision: true,
    });

    if any_open_van {
        for (player_entity, _, _) in pc.iter() {
            commands.entity(player_entity).insert(InTruck);
        }
    }

    if any_initialized {
        for (_, _player, p_transform) in pc.iter() {
            for mut cam_trans in camera.iter_mut() {
                cam_trans.translation = p_transform.translation;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, roomchanged_event);
}
