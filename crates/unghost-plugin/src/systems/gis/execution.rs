use bevy::prelude::*;
use bevy_replicon::prelude::ToClients;
use rand::prelude::*;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::Interactive;
use unbehavior_core::components::{InteractableByGhost, RoomStateDelta};
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use uncommon_app_core::random_seed;
use unghost_core::components::logic::interaction::{InteractionMotion, Locked};
use unghost_core::events::{
    GhostAudioMessage, GhostBreakerSparkRequest, GhostInteractionEvent, GhostInteractionType,
};
use uninteraction_core::events::{InteractionExecutionType, RoomChangedEvent};
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unmetrics_core::metrics::SendMetric;
use unspatial_core::position::Position;

use crate::metrics;
use crate::systems::ghost_ai::roar::emit_ghost_audio;

/// Enhanced destination validation with collision avoidance and path checking
fn validate_destination_enhanced(
    source: Position,
    destination: Position,
    board_topology: &BoardTopology,
    board_collision: &BoardCollisionField,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
) -> bool {
    let board_pos = destination.to_board_position();

    // 1. Check bounds
    if !board_pos.is_valid(board_topology.map_size) {
        return false;
    }

    // 2. Check collision - destination must be player_free (walkable)
    if let Some(idx) = board_pos.ndidx_checked(board_topology.map_size) {
        if let Some(collision_data) = board_collision.0.get(idx) {
            if !collision_data.player_free {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }

    // 3. Check for objects within 0.5 radius of destination
    for object_pos in q_objects.iter() {
        let distance = ((destination.x - object_pos.x).powi(2)
            + (destination.y - object_pos.y).powi(2))
        .sqrt();
        if distance < 0.5 {
            return false;
        }
    }

    // 4. Check for clear path from source to destination (except source position)
    if !has_clear_path(source, destination, board_topology, board_collision) {
        return false;
    }

    true
}

/// Checks if there's a clear path from source to destination
fn has_clear_path(
    source: Position,
    destination: Position,
    board_topology: &BoardTopology,
    board_collision: &BoardCollisionField,
) -> bool {
    let dx = destination.x - source.x;
    let dy = destination.y - source.y;
    let distance = (dx * dx + dy * dy).sqrt();

    if distance == 0.0 {
        return true; // Same position
    }

    // Number of steps to check along the path
    let steps = (distance * 4.0).ceil() as i32; // Check every 0.25 units

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let check_x = source.x + dx * t;
        let check_y = source.y + dy * t;

        let check_pos = Position {
            x: check_x,
            y: check_y,
            z: source.z,
            visual_priority: source.visual_priority,
        };

        let board_pos = check_pos.to_board_position();
        if let Some(idx) = board_pos.ndidx_checked(board_topology.map_size) {
            if let Some(collision_data) = board_collision.0.get(idx) {
                if !collision_data.player_free {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

/// Tries to find a valid destination with enhanced validation and retry logic
fn find_valid_destination_with_retry(
    source: Position,
    original_destination: Position,
    board_topology: &BoardTopology,
    board_collision: &BoardCollisionField,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    rng: &mut impl Rng,
    max_attempts: u32,
    search_radius: f32,
) -> Option<Position> {
    // First try the original destination
    if validate_destination_enhanced(
        source,
        original_destination,
        board_topology,
        board_collision,
        q_objects,
    ) {
        return Some(original_destination);
    }

    // If original destination fails, try nearby positions
    for _ in 0..max_attempts {
        let offset_x = rng.random_range(-search_radius..search_radius);
        let offset_y = rng.random_range(-search_radius..search_radius);

        let candidate_pos = Position {
            x: original_destination.x + offset_x,
            y: original_destination.y + offset_y,
            z: original_destination.z,
            visual_priority: original_destination.visual_priority,
        };

        if validate_destination_enhanced(
            source,
            candidate_pos,
            board_topology,
            board_collision,
            q_objects,
        ) {
            return Some(candidate_pos);
        }
    }

    None
}

/// Registers execution systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        bevy::prelude::Update,
        ghost_interaction_execution_system
            .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
    );
}

/// System that executes ghost interactions by processing GhostInteractionEvent
///
/// This system handles both legacy interactions (DoorSlam, Toggle) and new interactions
/// (Throw, Nudge, HauntedMove, Lock, TripBreaker) with appropriate state changes,
/// animations, and sound effects.
fn ghost_interaction_execution_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ev_ghost_interaction: MessageReader<GhostInteractionEvent>,
    mut local_audio: LocalAudioEmitter,
    mut ev_audio: MessageWriter<ToClients<GhostAudioMessage>>,
    mut ev_breaker_sparks: MessageWriter<GhostBreakerSparkRequest>,
    q_targets: Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    q_objects: Query<&Position, With<InteractableByGhost>>,
    mut ev_interaction_executor: MessageWriter<ExecuteInteractionEvent>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    board_topology: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    local_player_role: Option<Res<unreplicon_core::resources::LocalPlayerRole>>,
) {
    let measure = metrics::GIS_EXECUTION.time_measure();
    let has_local_player = local_player_role.is_some();
    let current_secs = time.elapsed_secs_f64();
    for event in ev_ghost_interaction.read() {
        // Minimal one-line log for each received interaction event
        if let Some(p) = event.destination {
            debug!(
                "GIS execution -> received {:?} for {:?} dest=({:.2}, {:.2}, {:.2})",
                event.interaction_type, event.target, p.x, p.y, p.z
            );
        } else {
            debug!(
                "GIS execution -> received {:?} for {:?}",
                event.interaction_type, event.target
            );
        }
        match event.interaction_type {
            GhostInteractionType::Toggle => {
                execute_toggle_interaction(
                    &mut ev_interaction_executor,
                    &mut ev_bdr,
                    &mut ev_room,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::DoorSlam => {
                execute_door_slam_interaction(
                    &mut local_audio,
                    &mut ev_audio,
                    &mut ev_interaction_executor,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                    has_local_player,
                );
            }

            GhostInteractionType::DoorCreak => {
                execute_door_creak_interaction(
                    &mut local_audio,
                    &mut ev_audio,
                    &mut ev_interaction_executor,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                    has_local_player,
                );
            }

            GhostInteractionType::Throw => {
                if let Some(destination) = event.destination {
                    execute_throw_interaction(
                        &mut commands,
                        &mut local_audio,
                        &mut ev_audio,
                        current_secs,
                        &q_targets,
                        &q_objects,
                        event.target,
                        destination,
                        &board_topology,
                        &board_collision,
                        has_local_player,
                    );
                } else {
                    error!(
                        "GIS execution -> Throw interaction for {:?} FAILED: missing destination (should not happen after target selection)",
                        event.target
                    );
                }
            }

            GhostInteractionType::Nudge => {
                execute_nudge_interaction(
                    &mut commands,
                    &mut local_audio,
                    &mut ev_audio,
                    current_secs,
                    &q_targets,
                    &q_objects,
                    event.target,
                    event.destination,
                    &board_topology,
                    &board_collision,
                    has_local_player,
                );
            }

            GhostInteractionType::HauntedMove => {
                if let Some(destination) = event.destination {
                    execute_haunted_move_interaction(
                        &mut commands,
                        &mut local_audio,
                        &mut ev_audio,
                        current_secs,
                        &q_targets,
                        &q_objects,
                        event.target,
                        destination,
                        &board_topology,
                        &board_collision,
                        has_local_player,
                    );
                } else {
                    error!(
                        "GIS execution -> HauntedMove interaction for {:?} FAILED: missing destination (should not happen after target selection)",
                        event.target
                    );
                }
            }

            GhostInteractionType::Lock => {
                execute_lock_interaction(
                    &mut commands,
                    &mut local_audio,
                    &mut ev_audio,
                    current_secs,
                    &q_targets,
                    event.target,
                    has_local_player,
                );
            }

            GhostInteractionType::TripBreaker => {
                execute_trip_breaker_interaction(
                    &mut local_audio,
                    &mut ev_audio,
                    &mut ev_breaker_sparks,
                    &mut ev_interaction_executor,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                    has_local_player,
                );
            }
        }
    }

    measure.end_ms();
}

fn broadcast_ghost_audio(
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    has_local_player: bool,
    sound_file: &str,
    volume: f32,
    position: Position,
) {
    emit_ghost_audio(
        sound_file.to_string(),
        volume,
        position,
        local_audio,
        ev_audio,
        has_local_player,
    );
}

/// Execute toggle interaction (lights, switches)
fn execute_toggle_interaction(
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    _ev_room: &mut MessageWriter<RoomChangedEvent>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
) {
    if let Ok((_, _position, _, _)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });
    } else {
        error!(
            "GIS execution -> Toggle interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute door slam interaction (fast door closure)
fn execute_door_slam_interaction(
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
    has_local_player: bool,
) {
    if let Ok((_behavior, position, _interactive, _room_state)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });
        broadcast_ghost_audio(
            local_audio,
            ev_audio,
            has_local_player,
            "sounds/door-close.ogg",
            1.5,
            *position,
        );
    } else {
        error!(
            "GIS execution -> DoorSlam interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute door creak interaction (slow door movement)
fn execute_door_creak_interaction(
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
    has_local_player: bool,
) {
    if let Ok((_behavior, position, _interactive, _room_state)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });
        broadcast_ghost_audio(
            local_audio,
            ev_audio,
            has_local_player,
            "sounds/door_creak_slow.ogg",
            0.7,
            *position,
        );
    } else {
        error!(
            "GIS execution -> DoorCreak interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute throw interaction (object flies through air)
fn execute_throw_interaction(
    commands: &mut Commands,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    current_secs: f64,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    target: Entity,
    destination: Position,
    board_topology: &BoardTopology,
    board_collision: &BoardCollisionField,
    has_local_player: bool,
) {
    if let Ok((_, current_position, _, _)) = q_targets.get(target) {
        let mut rng = random_seed::rng();

        // Try to find a valid destination with enhanced validation and retry logic
        if let Some(valid_destination) = find_valid_destination_with_retry(
            *current_position,
            destination,
            board_topology,
            board_collision,
            q_objects,
            &mut rng,
            30,  // max attempts
            2.0, // search radius
        ) {
            let motion = InteractionMotion::new_throw(
                *current_position,
                valid_destination,
                current_secs,
                0.5,
            );
            commands.entity(target).insert(motion);
            broadcast_ghost_audio(
                local_audio,
                ev_audio,
                has_local_player,
                "sounds/object_throw_generic.ogg",
                0.8,
                *current_position,
            );
        } else {
            warn!(
                "GIS execution -> Throw interaction for {:?} FAILED: could not find valid destination after 30 attempts (blocked paths, objects too close, or no valid tiles near ({:.2}, {:.2}, {:.2}))",
                target, destination.x, destination.y, destination.z
            );
        }
    } else {
        error!(
            "GIS execution -> Throw interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute nudge interaction (small object movement)
fn execute_nudge_interaction(
    commands: &mut Commands,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    current_secs: f64,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    target: Entity,
    destination: Option<Position>,
    board_topology: &BoardTopology,
    board_collision: &BoardCollisionField,
    has_local_player: bool,
) {
    if let Ok((_, current_position, _, _)) = q_targets.get(target) {
        // Handle destination validation if provided
        let final_destination = if let Some(dest) = destination {
            let mut rng = random_seed::rng();

            // Try to find a valid destination with enhanced validation and retry logic
            find_valid_destination_with_retry(
                *current_position,
                dest,
                board_topology,
                board_collision,
                q_objects,
                &mut rng,
                30,  // max attempts
                1.0, // smaller search radius for nudges
            )
        } else {
            None
        };

        let motion = if let Some(dest) = final_destination {
            InteractionMotion::new_nudge_to(*current_position, dest, current_secs, 0.6)
        } else if destination.is_some() {
            // Original destination was provided but validation failed
            warn!(
                "GIS execution -> Nudge interaction for {:?} FAILED: could not find valid destination after 30 attempts, falling back to local nudge",
                target
            );
            InteractionMotion::new_nudge(*current_position, current_secs, 0.45)
        } else {
            InteractionMotion::new_nudge(*current_position, current_secs, 0.45)
        };

        commands.entity(target).insert(motion);
        broadcast_ghost_audio(
            local_audio,
            ev_audio,
            has_local_player,
            "sounds/object_nudge_1.ogg",
            0.6,
            *current_position,
        );
    } else {
        error!(
            "GIS execution -> Nudge interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute haunted move interaction (slow object slide)
fn execute_haunted_move_interaction(
    commands: &mut Commands,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    current_secs: f64,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    target: Entity,
    destination: Position,
    board_topology: &BoardTopology,
    board_collision: &BoardCollisionField,
    has_local_player: bool,
) {
    if let Ok((_, current_position, _, _)) = q_targets.get(target) {
        let mut rng = random_seed::rng();

        // Try to find a valid destination with enhanced validation and retry logic
        if let Some(valid_destination) = find_valid_destination_with_retry(
            *current_position,
            destination,
            board_topology,
            board_collision,
            q_objects,
            &mut rng,
            30,  // max attempts
            2.5, // search radius for haunted moves
        ) {
            let motion = InteractionMotion::new_haunted_move(
                *current_position,
                valid_destination,
                current_secs,
                4.5,
            );
            commands.entity(target).insert(motion);
            broadcast_ghost_audio(
                local_audio,
                ev_audio,
                has_local_player,
                "sounds/object_drag_wood.ogg",
                0.9,
                *current_position,
            );
        } else {
            warn!(
                "GIS execution -> HauntedMove interaction for {:?} FAILED: could not find valid destination after 30 attempts (blocked paths, objects too close, or no valid tiles near ({:.2}, {:.2}, {:.2}))",
                target, destination.x, destination.y, destination.z
            );
        }
    } else {
        error!(
            "GIS execution -> HauntedMove interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute lock interaction (temporarily lock a door)
fn execute_lock_interaction(
    commands: &mut Commands,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    current_secs: f64,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
    has_local_player: bool,
) {
    // Check if the target entity exists and get its position for sound
    if let Ok((_, position, _, _)) = q_targets.get(target) {
        // Add a Locked component that expires 10 seconds from now
        commands
            .entity(target)
            .insert(Locked::new(current_secs, 10.0));
        broadcast_ghost_audio(
            local_audio,
            ev_audio,
            has_local_player,
            "sounds/door_lock_heavy.ogg",
            0.9,
            *position,
        );
    } else {
        error!(
            "GIS execution -> Lock interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute trip breaker interaction (turn off main power)
fn execute_trip_breaker_interaction(
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<GhostAudioMessage>>,
    ev_breaker_sparks: &mut MessageWriter<GhostBreakerSparkRequest>,
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
    has_local_player: bool,
) {
    if let Ok((_behavior, position, _interactive, _room_state)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });
        broadcast_ghost_audio(
            local_audio,
            ev_audio,
            has_local_player,
            "sounds/switch-on-2.ogg",
            1.0,
            *position,
        );

        ev_breaker_sparks.write(GhostBreakerSparkRequest {
            position: *position,
        });
    } else {
        error!(
            "GIS execution -> TripBreaker interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}
