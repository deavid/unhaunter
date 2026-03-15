pub mod interactivestuff;

use bevy::prelude::*;
use interactivestuff::InteractiveStuff;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::RoomState;
use unevents_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unevents_core::events::roomchanged::RoomStateSyncEvent;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unspatial_core::position::Position;

pub(crate) fn app_setup(app: &mut App) {
    // Authority-only: mutate Behavior state in response to interactions and room syncs.
    app.add_systems(
        Update,
        (interaction_event_handler, room_state_sync_system)
            .chain()
            .run_if(in_state(untypes_core::states::SimulationState::Ready))
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
    );
    // All nodes: rebuild board topology grids when any Behavior changes (including
    // changes arriving via bevy_replicon replication on pure clients).
    app.add_systems(
        Update,
        trigger_grid_rebuild_on_sync.run_if(in_state(untypes_core::states::SimulationState::Ready)),
    );
}

fn room_state_sync_system(
    mut ev_sync: MessageReader<RoomStateSyncEvent>,
    mut interactive_stuff: InteractiveStuff,
    q_interactables: Query<(Entity, &Position, &Behavior, &RoomState)>,
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
        debug!("Room state synchronization updated Behavior; rebuild is handled reactively.");
    }
}

fn trigger_grid_rebuild_on_sync(
    q_changed_behavior: Query<(), Changed<Behavior>>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
) {
    if !q_changed_behavior.is_empty() {
        ev_bdr.write(BoardTopologyToRebuild {
            lighting: true,
            collision: true,
        });
    }
}

fn interaction_event_handler(
    mut ev_reader: MessageReader<ExecuteInteractionEvent>,
    mut interactive_stuff: InteractiveStuff,
    q_interactive: Query<(
        Option<&Interactive>,
        &Behavior,
        Option<&RoomState>,
        &Position,
    )>,
    mut ev_room_sync: MessageWriter<RoomStateSyncEvent>,
) {
    for ev in ev_reader.read() {
        if let Ok((interactive, behavior, room_state, pos)) = q_interactive.get(ev.entity) {
            if interactive_stuff.execute_interaction(
                ev.entity,
                pos,
                interactive,
                behavior,
                room_state,
                ev.ietype.clone(),
                ev.force_tuid,
            ) {
                debug!("Interaction successful, scheduling room sync");
                ev_room_sync.write(RoomStateSyncEvent);
            }
        } else {
            warn!(
                "Could not find interactive components for entity {:?}",
                ev.entity
            );
        }
    }
}

