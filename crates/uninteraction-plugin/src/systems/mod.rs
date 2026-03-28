pub mod interactivestuff;

use bevy::picking::events::{Out, Over, Pointer};
use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use interactivestuff::InteractiveStuff;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::Interactive;
use unbehavior_core::components::{FloorItemCollidable, RoomStateDelta, TmxEntityId};
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use uninteraction_core::events::RoomStateSyncEvent;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unplayer_core::components::{MainPlayer, PlayerSpectating};
use unspatial_core::position::Position;

pub(crate) fn app_setup(app: &mut App) {
    app.replicate::<TmxEntityId>();
    app.replicate::<Behavior>();
    app.replicate::<FloorItemCollidable>();
    app.replicate::<Toggleable>();

    // Authority-only: mutate Behavior state in response to interactions and room syncs.
    app.add_systems(
        Update,
        (interaction_event_handler, room_state_sync_system)
            .chain()
            .run_if(in_state(unmission_core::types::SimulationState::Ready))
            .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
    );
    // All nodes: rebuild board topology grids when any Behavior changes (including
    // changes arriving via bevy_replicon replication on pure clients).
    app.add_systems(
        Update,
        trigger_grid_rebuild_on_sync
            .run_if(in_state(unmission_core::types::SimulationState::Ready)),
    );
    // Mouse hover feedback: mark interactive objects as hovered/unhovered
    app.add_systems(
        Update,
        (mouse_over_interactive_system, mouse_out_interactive_system)
            .run_if(in_state(unorchestrator_core::UIContextState::InGame)),
    );
}

fn mouse_over_interactive_system(
    mut events: MessageReader<Pointer<Over>>,
    mut q_interactive: Query<&mut Interactive>,
    q_spectator: Query<(), (With<MainPlayer>, With<PlayerSpectating>)>,
) {
    let is_spectator = !q_spectator.is_empty();
    for event in events.read() {
        if is_spectator {
            continue;
        }
        if let Ok(mut interactive) = q_interactive.get_mut(event.entity) {
            interactive.hovered = true;
        }
    }
}

fn mouse_out_interactive_system(
    mut events: MessageReader<Pointer<Out>>,
    mut q_interactive: Query<&mut Interactive>,
    q_spectator: Query<(), (With<MainPlayer>, With<PlayerSpectating>)>,
) {
    let is_spectator = !q_spectator.is_empty();
    for event in events.read() {
        if is_spectator {
            continue;
        }
        if let Ok(mut interactive) = q_interactive.get_mut(event.entity) {
            interactive.hovered = false;
        }
    }
}

fn room_state_sync_system(
    mut ev_sync: MessageReader<RoomStateSyncEvent>,
    mut interactive_stuff: InteractiveStuff,
    q_interactables: Query<(Entity, &Position, &Behavior, &RoomStateDelta)>,
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
        Option<&RoomStateDelta>,
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
