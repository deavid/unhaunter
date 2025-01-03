use super::{GCameraArena, GameConfig};
use crate::uncore_behavior::component::RoomState;
use crate::uncore_behavior::Behavior;
use crate::board::{
    self, BoardDataToRebuild,
};
use crate::player::{InteractiveStuff, PlayerSprite};
use crate::root;
use bevy::prelude::*;

/// Event triggered when the player enters a new room or when a significant
/// room-related change occurs.
///
/// This event is used to trigger actions like opening the van UI or updating the
/// state of interactive objects based on the room's current state.
#[derive(Clone, Debug, Default, Event)]
pub struct RoomChangedEvent {
    /// Set to `true` if the event is triggered during level initialization.
    pub initialize: bool,
    /// Set to `true` if the van UI should be opened automatically (e.g., when the
    /// player returns to the starting area).
    pub open_van: bool,
}

impl RoomChangedEvent {
    /// Creates a new `RoomChangedEvent` specifically for level initialization.
    ///
    /// The `initialize` flag is set to `true`, and the `open_van` flag is set based on
    /// the given value.
    pub fn init(open_van: bool) -> Self {
        Self {
            initialize: true,
            open_van,
        }
    }
}

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
pub fn roomchanged_event(
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    mut ev_room: EventReader<RoomChangedEvent>,
    mut interactive_stuff: InteractiveStuff,
    interactables: Query<(Entity, &board::Position, &Behavior, &RoomState), Without<PlayerSprite>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform), Without<GCameraArena>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
) {
    let Some(ev) = ev_room.read().next() else {
        return;
    };
    for (entity, item_pos, behavior, room_state) in interactables.iter() {
        let changed = interactive_stuff.execute_interaction(
            entity,
            item_pos,
            None,
            behavior,
            Some(room_state),
            InteractionExecutionType::ReadRoomState,
        );
        if changed {
            // dbg!(&behavior);
        }
    }
    ev_bdr.send(BoardDataToRebuild {
        lighting: true,
        collision: true,
    });
    if ev.open_van {
        interactive_stuff
            .game_next_state
            .set(root::GameState::Truck);
    }
    if ev.initialize {
        for (player, p_transform) in pc.iter() {
            if player.id != gc.player_id {
                continue;
            }
            for mut cam_trans in camera.iter_mut() {
                cam_trans.translation = p_transform.translation;
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InteractionExecutionType {
    ChangeState,
    ReadRoomState,
}

pub fn app_setup(app: &mut App) {
    app.add_event::<RoomChangedEvent>()
        .add_systems(Update, roomchanged_event);
}
