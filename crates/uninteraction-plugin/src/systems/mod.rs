pub mod interactivestuff;

use bevy::prelude::*;
use interactivestuff::InteractiveStuff;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::RoomState;
use unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unevents_core::events::roomchanged::RoomChangedEvent;
use uninteraction_core::interaction::{Authority, ExecuteInteractionEvent};
use unspatial_core::position::Position;
use untypes_core::cli::{CliOptions, is_host};

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, interaction_event_handler);
}

fn interaction_event_handler(
    mut ev_reader: MessageReader<ExecuteInteractionEvent>,
    mut interactive_stuff: InteractiveStuff,
    cli: Res<CliOptions>,
    q_interactive: Query<(
        Option<&Interactive>,
        &Behavior,
        Option<&RoomState>,
        &Position,
    )>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
) {
    let authority = if is_host(cli) {
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
                    ev_room.write(RoomChangedEvent::default());
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
