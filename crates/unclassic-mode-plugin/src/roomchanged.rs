use bevy::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::components::RoomState;
use unengine_core::GCameraArena;
use unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unevents_core::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unspatial_core::position::Position;
use untypes_core::states::GameState;

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
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
    mut ev_room: MessageReader<RoomChangedEvent>,
    mut ev_interaction: MessageWriter<ExecuteInteractionEvent>,
    interactables: Query<(Entity, &Position, &Behavior, &RoomState), Without<PlayerSprite>>,
    pc: Query<(&PlayerSprite, &Transform), (Without<GCameraArena>, With<MainPlayer>)>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
    mut game_next_state: ResMut<NextState<GameState>>,
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

    for (entity, _, _, _) in interactables.iter() {
        ev_interaction.write(ExecuteInteractionEvent {
            entity,
            ietype: InteractionExecutionType::ReadRoomState,
            force_tuid: None,
        });
    }

    ev_bdr.write(BoardTopologyToRebuild {
        lighting: true,
        collision: true,
    });

    if any_open_van {
        game_next_state.set(GameState::Truck);
    }

    if any_initialized {
        for (_player, p_transform) in pc.iter() {
            for mut cam_trans in camera.iter_mut() {
                cam_trans.translation = p_transform.translation;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<RoomChangedEvent>()
        .add_systems(Update, roomchanged_event);
}
