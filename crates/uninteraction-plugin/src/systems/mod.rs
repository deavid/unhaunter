pub mod interactivestuff;

use bevy::prelude::*;
use interactivestuff::InteractiveStuff;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::RoomState;
use unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unevents_core::events::roomchanged::RoomStateSyncEvent;
use uninteraction_core::interaction::{Authority, ExecuteInteractionEvent};
use unspatial_core::position::Position;
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (interaction_event_handler, room_state_sync_system)
            .chain()
            .run_if(in_state(untypes_core::states::SimulationState::Ready)),
    );
}

fn room_state_sync_system(
    mut ev_sync: MessageReader<RoomStateSyncEvent>,
    mut interactive_stuff: InteractiveStuff,
    q_interactables: Query<(Entity, &Position, &Behavior, &RoomState)>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
) {
    if ev_sync.read().next().is_none() {
        return;
    }

    let mut changed = false;
    for (entity, pos, behavior, room_state) in q_interactables.iter() {
        if interactive_stuff.synchronize_entity(entity, pos, behavior, room_state) {
            changed = true;
        }
    }

    if changed {
        debug!("Room state synchronization triggered a board topology rebuild.");
        ev_bdr.write(BoardTopologyToRebuild {
            lighting: true,
            collision: true,
        });
    }
}

fn interaction_event_handler(
    mut ev_reader: MessageReader<ExecuteInteractionEvent>,
    mut interactive_stuff: InteractiveStuff,
    authority_role: Option<Res<untypes_core::roles::AuthorityRole>>,
    q_interactive: Query<(
        Option<&Interactive>,
        &Behavior,
        Option<&RoomState>,
        &Position,
    )>,
    mut ev_room_sync: MessageWriter<RoomStateSyncEvent>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
) {
    let authority = if authority_role.is_some() {
        Authority::Host
    } else {
        Authority::Client
    };
    for ev in ev_reader.read() {
        if let Ok((interactive, behavior, room_state, pos)) = q_interactive.get(ev.entity) {
            if interactive_stuff.execute_interaction(
                ev.entity,
                pos,
                interactive,
                behavior,
                room_state,
                ev.ietype.clone(),
                authority,
                ev.force_tuid,
            ) {
                debug!(
                    "Interaction successful, rewriting board topology (authority={:?})",
                    authority
                );
                if authority == Authority::Host {
                    ev_room_sync.write(RoomStateSyncEvent);
                }
                ev_bdr.write(BoardTopologyToRebuild {
                    lighting: true,
                    collision: true,
                });
            }
        } else {
            warn!(
                "Could not find interactive components for entity {:?}",
                ev.entity
            );
        }
    }
}
