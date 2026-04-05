pub mod interactivestuff;

use bevy::picking::events::{Out, Over, Pointer};
use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt, Channel, ClientMessageAppExt, FromClient};
use interactivestuff::InteractiveStuff;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::Interactive;
use unbehavior_core::components::{FloorItemCollidable, RoomStateDelta, TmxEntityId};
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use uninteraction_core::events::InteractionRequestMessage;
use uninteraction_core::events::RoomStateSyncEvent;
use uninteraction_core::hover::HoverState;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unplayer_core::components::{MainPlayer, PlayerSpectating};
use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::position::Position;

pub(crate) fn app_setup_core(app: &mut App) {
    app.add_client_message::<InteractionRequestMessage>(Channel::Ordered);
    app.replicate::<TmxEntityId>();
    app.replicate::<Behavior>();
    app.replicate::<FloorItemCollidable>();
    app.replicate::<Toggleable>();

    // Authority-only: mutate Behavior state in response to interactions and room syncs.
    app.add_systems(
        Update,
        (
            handle_interaction_request,
            interaction_event_handler,
            room_state_sync_system,
        )
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
}

pub(crate) fn app_setup_client(app: &mut App) {
    // Mouse hover feedback: mark interactive objects as hovered/unhovered
    app.add_systems(
        Update,
        (
            ensure_hover_state,
            mouse_over_interactive_system,
            mouse_out_interactive_system,
        )
            .chain()
            .run_if(in_state(unorchestrator_core::UIContextState::InGame)),
    );
}

fn ensure_hover_state(
    mut commands: Commands,
    q_interactive: Query<Entity, (With<Interactive>, Without<HoverState>)>,
) {
    for entity in q_interactive.iter() {
        commands.entity(entity).insert(HoverState::default());
    }
}

fn mouse_over_interactive_system(
    mut events: MessageReader<Pointer<Over>>,
    mut q_hover: Query<&mut HoverState>,
    q_spectator: Query<(), (With<MainPlayer>, With<PlayerSpectating>)>,
) {
    let is_spectator = !q_spectator.is_empty();
    for event in events.read() {
        if is_spectator {
            continue;
        }
        if let Ok(mut hover) = q_hover.get_mut(event.entity) {
            hover.is_hovered = true;
        }
    }
}

fn mouse_out_interactive_system(
    mut events: MessageReader<Pointer<Out>>,
    mut q_hover: Query<&mut HoverState>,
    q_spectator: Query<(), (With<MainPlayer>, With<PlayerSpectating>)>,
) {
    let is_spectator = !q_spectator.is_empty();
    for event in events.read() {
        if is_spectator {
            continue;
        }
        if let Ok(mut hover) = q_hover.get_mut(event.entity) {
            hover.is_hovered = false;
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

fn handle_interaction_request(
    mut reader: MessageReader<FromClient<InteractionRequestMessage>>,
    q_interactive: Query<(Entity, &MapEntityFieldBPos), With<Interactive>>,
    mut ev_interact: MessageWriter<ExecuteInteractionEvent>,
) {
    for msg in reader.read() {
        let target_bpos = BoardPosition {
            x: msg.message.position[0] as i64,
            y: msg.message.position[1] as i64,
            z: msg.message.position[2] as i64,
        };

        let found = q_interactive
            .iter()
            .find(|(_, bpos)| bpos.0 == target_bpos)
            .map(|(entity, _)| entity);

        if let Some(entity) = found {
            info!(
                "SERVER: Validated door interaction at {:?} from client {:?}",
                target_bpos, msg.client_id
            );
            info!(
                "SERVER: Firing ExecuteInteractionEvent for entity {:?} (ietype={:?}, force_tuid={:?})",
                entity, msg.message.ietype, msg.message.force_tuid
            );
            ev_interact.write(ExecuteInteractionEvent {
                entity,
                ietype: msg.message.ietype.clone(),
                force_tuid: msg.message.force_tuid,
            });
        } else {
            warn!(
                "SERVER: Failed to find interactive entity at {:?}",
                target_bpos
            );
        }
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
