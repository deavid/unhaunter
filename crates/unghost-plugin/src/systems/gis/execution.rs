use bevy::prelude::*;
use rand::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::{InteractableByGhost, RoomStateDelta};
use unboard_core::events::board_topology_rebuild::BoardTopologyToRebuild;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unfoundation_core::random_seed;
use unghost_core::events::{GhostInteractionEvent, GhostInteractionType};
use uninteraction_core::events::{InteractionExecutionType, RoomChangedEvent};
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::messages::HostMovableMotionEvent;
use unaudiospatial_core::events::SoundEvent;
use unspatial_core::position::Position;

use crate::metrics;

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

use crate::systems::gis::visual_effects;

use crate::components::interaction::{Locked, Tween, TweenEase};

/// Registers execution systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        bevy::prelude::Update,
        (ghost_interaction_execution_system, watch_tween_insertions)
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
    );
}

/// System that executes ghost interactions by processing GhostInteractionEvent
///
/// This system handles both legacy interactions (DoorSlam, Toggle) and new interactions
/// (Throw, Nudge, HauntedMove, Lock, TripBreaker) with appropriate state changes,
/// animations, and sound effects.
fn ghost_interaction_execution_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_ghost_interaction: MessageReader<GhostInteractionEvent>,
    q_targets: Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    q_objects: Query<&Position, With<InteractableByGhost>>,
    mut ev_interaction_executor: MessageWriter<ExecuteInteractionEvent>,
    mut ev_sound: MessageWriter<SoundEvent>,
    mut ev_bdr: MessageWriter<BoardTopologyToRebuild>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    board_topology: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    local_player_role: Option<Res<untypes_core::roles::LocalPlayerRole>>,
) {
    let measure = metrics::GIS_EXECUTION.time_measure();
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
                    &mut ev_interaction_executor,
                    &mut ev_sound,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::DoorCreak => {
                execute_door_creak_interaction(
                    &mut ev_interaction_executor,
                    &mut ev_sound,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::Throw => {
                if let Some(destination) = event.destination {
                    execute_throw_interaction(
                        &mut commands,
                        &mut ev_sound,
                        &q_targets,
                        &q_objects,
                        event.target,
                        destination,
                        &board_topology,
                        &board_collision,
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
                    &mut ev_sound,
                    &q_targets,
                    &q_objects,
                    event.target,
                    event.destination,
                    &board_topology,
                    &board_collision,
                );
            }

            GhostInteractionType::HauntedMove => {
                if let Some(destination) = event.destination {
                    execute_haunted_move_interaction(
                        &mut commands,
                        &mut ev_sound,
                        &q_targets,
                        &q_objects,
                        event.target,
                        destination,
                        &board_topology,
                        &board_collision,
                    );
                } else {
                    error!(
                        "GIS execution -> HauntedMove interaction for {:?} FAILED: missing destination (should not happen after target selection)",
                        event.target
                    );
                }
            }

            GhostInteractionType::Lock => {
                execute_lock_interaction(&mut commands, &mut ev_sound, &q_targets, event.target);
            }

            GhostInteractionType::TripBreaker => {
                execute_trip_breaker_interaction(
                    &mut commands,
                    &asset_server,
                    &mut ev_interaction_executor,
                    &mut ev_sound,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                    local_player_role.as_deref(),
                );
            }
        }
    }

    measure.end_ms();
}

/// Server-only: emit `HostMovableMotionEvent` whenever a new `Tween` is inserted
/// on a map entity (thrown, nudged, or haunted-moved by the ghost).
///
/// `unreplicon-plugin`'s `broadcast_movable_motion` system reads this event and
/// forwards the animation parameters to all connected join clients as
/// `MovableMotionBroadcast`, which they replay locally via `apply_remote_movable_motion`.
fn watch_tween_insertions(
    q_new_tweens: Query<(Entity, &Tween), Added<Tween>>,
    mut ev_host_movable: MessageWriter<HostMovableMotionEvent>,
) {
    for (entity, tween) in q_new_tweens.iter() {
        let ease = match tween.ease_fn {
            TweenEase::Linear => 0u8,
            TweenEase::ParabolicArc => 1u8,
            TweenEase::SineEaseOut => 2u8,
        };
        ev_host_movable.write(HostMovableMotionEvent {
            entity,
            start: [
                tween.start_pos.x,
                tween.start_pos.y,
                tween.start_pos.z,
                tween.start_pos.visual_priority,
            ],
            end: [
                tween.end_pos.x,
                tween.end_pos.y,
                tween.end_pos.z,
                tween.end_pos.visual_priority,
            ],
            duration: tween.timer.duration().as_secs_f32(),
            ease,
        });
    }
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
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    ev_sound: &mut MessageWriter<SoundEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
) {
    if let Ok((_behavior, position, _interactive, _room_state)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });

        ev_sound.write(SoundEvent {
            sound_file: "sounds/door-close.ogg".to_string(),
            volume: 1.5, // Louder than normal door close to simulate slam
            position: Some(*position),
            broadcast: true,
        });
    } else {
        error!(
            "GIS execution -> DoorSlam interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute door creak interaction (slow door movement)
fn execute_door_creak_interaction(
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    ev_sound: &mut MessageWriter<SoundEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
) {
    if let Ok((_behavior, position, _interactive, _room_state)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });

        // Play door creak sound effect
        ev_sound.write(SoundEvent {
            sound_file: "sounds/door_creak_slow.ogg".to_string(),
            volume: 0.7,
            position: Some(*position),
            broadcast: true,
        });
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
    ev_sound: &mut MessageWriter<SoundEvent>,
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
            // Create a tween animation for the throw
            let tween = Tween::new_throw(*current_position, valid_destination, 0.5);
            commands.entity(target).insert(tween);

            // Play throw sound effect
            ev_sound.write(SoundEvent {
                sound_file: "sounds/object_throw_generic.ogg".to_string(),
                volume: 0.8,
                position: Some(*current_position),
                broadcast: true,
            });
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
    ev_sound: &mut MessageWriter<SoundEvent>,
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

        // Create appropriate tween based on destination availability
        let tween = if let Some(dest) = final_destination {
            Tween::new_nudge_to(*current_position, dest, 0.6)
        } else if destination.is_some() {
            // Original destination was provided but validation failed
            warn!(
                "GIS execution -> Nudge interaction for {:?} FAILED: could not find valid destination after 30 attempts, falling back to local nudge",
                target
            );
            Tween::new_nudge(*current_position, 0.45)
        } else {
            // No destination provided, use local nudge
            Tween::new_nudge(*current_position, 0.45)
        };

        commands.entity(target).insert(tween);

        // Play nudge sound effect
        ev_sound.write(SoundEvent {
            sound_file: "sounds/object_nudge_1.ogg".to_string(),
            volume: 0.6,
            position: Some(*current_position),
            broadcast: true,
        });
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
    ev_sound: &mut MessageWriter<SoundEvent>,
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
            // Create a slow haunted movement animation
            let tween = Tween::new_haunted_move(*current_position, valid_destination, 4.5);
            commands.entity(target).insert(tween);

            // Play haunted move sound effect
            ev_sound.write(SoundEvent {
                sound_file: "sounds/object_drag_wood.ogg".to_string(),
                volume: 0.9,
                position: Some(*current_position),
                broadcast: true,
            });
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
    ev_sound: &mut MessageWriter<SoundEvent>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
) {
    // Check if the target entity exists and get its position for sound
    if let Ok((_, position, _, _)) = q_targets.get(target) {
        // Add a locked component with a 10-second timer
        let lock_timer = Timer::from_seconds(10.0, TimerMode::Once);
        commands.entity(target).insert(Locked(lock_timer));

        // Play door lock sound effect
        ev_sound.write(SoundEvent {
            sound_file: "sounds/door_lock_heavy.ogg".to_string(),
            volume: 0.9,
            position: Some(*position),
            broadcast: true,
        });
    } else {
        error!(
            "GIS execution -> Lock interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute trip breaker interaction (turn off main power)
fn execute_trip_breaker_interaction(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    ev_interaction_executor: &mut MessageWriter<ExecuteInteractionEvent>,
    ev_sound: &mut MessageWriter<SoundEvent>,
    _ev_bdr: &mut MessageWriter<BoardTopologyToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomStateDelta>,
    )>,
    target: Entity,
    local_player_role: Option<&untypes_core::roles::LocalPlayerRole>,
) {
    if let Ok((_behavior, position, _interactive, _room_state)) = q_targets.get(target) {
        ev_interaction_executor.write(ExecuteInteractionEvent {
            entity: target,
            ietype: InteractionExecutionType::ChangeState,
            force_tuid: None,
        });

        // Play breaker trip sound effect
        ev_sound.write(SoundEvent {
            sound_file: "sounds/switch-on-2.ogg".to_string(),
            volume: 1.0,
            position: Some(*position),
            broadcast: true,
        });

        // Spawn electrical sparks visual effect
        if local_player_role.is_some() {
            visual_effects::spawn_electrical_sparks(commands, asset_server, *position);
        }
    } else {
        error!(
            "GIS execution -> TripBreaker interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}
