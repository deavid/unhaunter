use bevy::prelude::*;
use rand::Rng;
use uncore_board::behavior::Behavior;
use uncore_board::behavior::component::{InteractableByGhost, Interactive, RoomState};
use uncore_board::resources::board_data::BoardData;
use uncore_events::events::board_data_rebuild::BoardDataToRebuild;
use uncore_events::events::ghost_interaction::{GhostInteractionEvent, GhostInteractionType};
use uncore_events::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use uncore_events::events::sound::SoundEvent;
use uncore_foundation::random_seed;
use uninteraction_core::interactivestuff::InteractiveStuff;
use unspatial_core::Position;

/// Enhanced destination validation with collision avoidance and path checking
fn validate_destination_enhanced(
    source: Position,
    destination: Position,
    board_data: &BoardData,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
) -> bool {
    let board_pos = destination.to_board_position();

    // 1. Check bounds
    if !board_pos.is_valid(board_data.map_size) {
        return false;
    }

    // 2. Check collision - destination must be player_free (walkable)
    if let Some(idx) = board_pos.ndidx_checked(board_data.map_size) {
        if let Some(collision_data) = board_data.collision_field.get(idx) {
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
    if !has_clear_path(source, destination, board_data) {
        return false;
    }

    true
}

/// Checks if there's a clear path from source to destination
fn has_clear_path(source: Position, destination: Position, board_data: &BoardData) -> bool {
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
            global_z: source.global_z,
        };

        let board_pos = check_pos.to_board_position();
        if let Some(idx) = board_pos.ndidx_checked(board_data.map_size) {
            if let Some(collision_data) = board_data.collision_field.get(idx) {
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
    board_data: &BoardData,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    rng: &mut impl Rng,
    max_attempts: u32,
    search_radius: f32,
) -> Option<Position> {
    // First try the original destination
    if validate_destination_enhanced(source, original_destination, board_data, q_objects) {
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
            global_z: original_destination.global_z,
        };

        if validate_destination_enhanced(source, candidate_pos, board_data, q_objects) {
            return Some(candidate_pos);
        }
    }

    None
}

use crate::systems::gis::visual_effects;

use crate::components::interaction::{Locked, Tween};

/// Registers execution systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(bevy::prelude::Update, ghost_interaction_execution_system);
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
        Option<&RoomState>,
    )>,
    q_objects: Query<&Position, With<InteractableByGhost>>,
    mut interactive_stuff: InteractiveStuff,
    mut ev_bdr: MessageWriter<BoardDataToRebuild>,
    mut ev_room: MessageWriter<RoomChangedEvent>,
    board_data: Res<BoardData>,
) {
    for event in ev_ghost_interaction.read() {
        // Minimal one-line log for each received interaction event
        if let Some(p) = event.destination {
            info!(
                "GIS execution -> received {:?} for {:?} dest=({:.2}, {:.2}, {:.2})",
                event.interaction_type, event.target, p.x, p.y, p.z
            );
        } else {
            info!(
                "GIS execution -> received {:?} for {:?}",
                event.interaction_type, event.target
            );
        }
        match event.interaction_type {
            GhostInteractionType::Toggle => {
                execute_toggle_interaction(
                    &mut interactive_stuff,
                    &mut ev_bdr,
                    &mut ev_room,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::DoorSlam => {
                execute_door_slam_interaction(
                    &mut interactive_stuff,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::DoorCreak => {
                execute_door_creak_interaction(
                    &mut interactive_stuff,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::Throw => {
                if let Some(destination) = event.destination {
                    execute_throw_interaction(
                        &mut commands,
                        &mut interactive_stuff,
                        &q_targets,
                        &q_objects,
                        event.target,
                        destination,
                        &board_data,
                    );
                } else {
                    warn!(
                        "GIS execution -> Throw interaction for {:?} FAILED: missing destination (should not happen after target selection)",
                        event.target
                    );
                }
            }

            GhostInteractionType::Nudge => {
                execute_nudge_interaction(
                    &mut commands,
                    &mut interactive_stuff,
                    &q_targets,
                    &q_objects,
                    event.target,
                    event.destination,
                    &board_data,
                );
            }

            GhostInteractionType::HauntedMove => {
                if let Some(destination) = event.destination {
                    execute_haunted_move_interaction(
                        &mut commands,
                        &mut interactive_stuff,
                        &q_targets,
                        &q_objects,
                        event.target,
                        destination,
                        &board_data,
                    );
                } else {
                    warn!(
                        "GIS execution -> HauntedMove interaction for {:?} FAILED: missing destination (should not happen after target selection)",
                        event.target
                    );
                }
            }

            GhostInteractionType::Lock => {
                execute_lock_interaction(
                    &mut commands,
                    &mut interactive_stuff,
                    &q_targets,
                    event.target,
                );
            }

            GhostInteractionType::TripBreaker => {
                execute_trip_breaker_interaction(
                    &mut commands,
                    &asset_server,
                    &mut interactive_stuff,
                    &mut ev_bdr,
                    &q_targets,
                    event.target,
                );
            }
        }
    }
}

/// Execute toggle interaction (lights, switches)
fn execute_toggle_interaction(
    interactive_stuff: &mut InteractiveStuff,
    ev_bdr: &mut MessageWriter<BoardDataToRebuild>,
    ev_room: &mut MessageWriter<RoomChangedEvent>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    target: Entity,
) {
    // Get the behavior component to execute the interaction
    if let Ok((behavior, position, interactive, room_state)) = q_targets.get(target) {
        // Execute the toggle interaction using the existing InteractiveStuff system
        let changed = interactive_stuff.execute_interaction(
            target,
            position,
            interactive,
            behavior,
            room_state,
            InteractionExecutionType::ChangeState,
        );

        // If the interaction changed the state, trigger room update
        if changed {
            ev_room.write(RoomChangedEvent::default());
        }

        // Rebuild lighting and collision data
        ev_bdr.write(BoardDataToRebuild {
            lighting: true,
            collision: true,
        });
    } else {
        warn!(
            "GIS execution -> Toggle interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute door slam interaction (fast door closure)
fn execute_door_slam_interaction(
    interactive_stuff: &mut InteractiveStuff,
    ev_bdr: &mut MessageWriter<BoardDataToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    target: Entity,
) {
    // Get the behavior component to execute the interaction
    if let Ok((behavior, position, _interactive, room_state)) = q_targets.get(target) {
        // Execute the door slam using the existing InteractiveStuff system
        interactive_stuff.execute_interaction(
            target,
            position,
            None, // suppress default sound; we play a custom slam below
            behavior,
            room_state,
            InteractionExecutionType::ChangeState,
        );

        // Play door slam sound effect (using door-close.ogg with higher volume)
        interactive_stuff.sound_events.write(SoundEvent {
            sound_file: "sounds/door-close.ogg".to_string(),
            volume: 1.5, // Louder than normal door close to simulate slam
            position: Some(*position),
        });

        // Rebuild lighting and collision data
        ev_bdr.write(BoardDataToRebuild {
            lighting: true,
            collision: true,
        });
    } else {
        warn!(
            "GIS execution -> DoorSlam interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute door creak interaction (slow door movement)
fn execute_door_creak_interaction(
    interactive_stuff: &mut InteractiveStuff,
    ev_bdr: &mut MessageWriter<BoardDataToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    target: Entity,
) {
    // Get the behavior component to execute the interaction
    if let Ok((behavior, position, _interactive, room_state)) = q_targets.get(target) {
        // Execute the door creak using the existing InteractiveStuff system
        interactive_stuff.execute_interaction(
            target,
            position,
            None, // suppress default sound; we play a custom creak below
            behavior,
            room_state,
            InteractionExecutionType::ChangeState,
        );

        // Play door creak sound effect
        interactive_stuff.sound_events.write(SoundEvent {
            sound_file: "sounds/door_creak_slow.ogg".to_string(),
            volume: 0.7,
            position: Some(*position),
        });

        // Rebuild lighting and collision data
        ev_bdr.write(BoardDataToRebuild {
            lighting: true,
            collision: true,
        });
    } else {
        warn!(
            "GIS execution -> DoorCreak interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute throw interaction (object flies through air)
fn execute_throw_interaction(
    commands: &mut Commands,
    interactive_stuff: &mut InteractiveStuff,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    target: Entity,
    destination: Position,
    board_data: &BoardData,
) {
    if let Ok((_, current_position, _, _)) = q_targets.get(target) {
        let mut rng = random_seed::rng();

        // Try to find a valid destination with enhanced validation and retry logic
        if let Some(valid_destination) = find_valid_destination_with_retry(
            *current_position,
            destination,
            board_data,
            q_objects,
            &mut rng,
            30,  // max attempts
            2.0, // search radius
        ) {
            // Create a tween animation for the throw
            let tween = Tween::new_throw(*current_position, valid_destination, 0.5);
            commands.entity(target).insert(tween);

            // Play throw sound effect
            interactive_stuff.sound_events.write(SoundEvent {
                sound_file: "sounds/object_throw_generic.ogg".to_string(),
                volume: 0.8,
                position: Some(*current_position),
            });
        } else {
            warn!(
                "GIS execution -> Throw interaction for {:?} FAILED: could not find valid destination after 30 attempts (blocked paths, objects too close, or no valid tiles near ({:.2}, {:.2}, {:.2}))",
                target, destination.x, destination.y, destination.z
            );
        }
    } else {
        warn!(
            "GIS execution -> Throw interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute nudge interaction (small object movement)
fn execute_nudge_interaction(
    commands: &mut Commands,
    interactive_stuff: &mut InteractiveStuff,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    target: Entity,
    destination: Option<Position>,
    board_data: &BoardData,
) {
    if let Ok((_, current_position, _, _)) = q_targets.get(target) {
        // Handle destination validation if provided
        let final_destination = if let Some(dest) = destination {
            let mut rng = random_seed::rng();

            // Try to find a valid destination with enhanced validation and retry logic
            find_valid_destination_with_retry(
                *current_position,
                dest,
                board_data,
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
        interactive_stuff.sound_events.write(SoundEvent {
            sound_file: "sounds/object_nudge_1.ogg".to_string(),
            volume: 0.6,
            position: Some(*current_position),
        });
    } else {
        warn!(
            "GIS execution -> Nudge interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute haunted move interaction (slow object slide)
fn execute_haunted_move_interaction(
    commands: &mut Commands,
    interactive_stuff: &mut InteractiveStuff,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    q_objects: &Query<&Position, With<InteractableByGhost>>,
    target: Entity,
    destination: Position,
    board_data: &BoardData,
) {
    if let Ok((_, current_position, _, _)) = q_targets.get(target) {
        let mut rng = random_seed::rng();

        // Try to find a valid destination with enhanced validation and retry logic
        if let Some(valid_destination) = find_valid_destination_with_retry(
            *current_position,
            destination,
            board_data,
            q_objects,
            &mut rng,
            30,  // max attempts
            2.5, // search radius for haunted moves
        ) {
            // Create a slow haunted movement animation
            let tween = Tween::new_haunted_move(*current_position, valid_destination, 4.5);
            commands.entity(target).insert(tween);

            // Play haunted move sound effect
            interactive_stuff.sound_events.write(SoundEvent {
                sound_file: "sounds/object_drag_wood.ogg".to_string(),
                volume: 0.9,
                position: Some(*current_position),
            });
        } else {
            warn!(
                "GIS execution -> HauntedMove interaction for {:?} FAILED: could not find valid destination after 30 attempts (blocked paths, objects too close, or no valid tiles near ({:.2}, {:.2}, {:.2}))",
                target, destination.x, destination.y, destination.z
            );
        }
    } else {
        warn!(
            "GIS execution -> HauntedMove interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute lock interaction (temporarily lock a door)
fn execute_lock_interaction(
    commands: &mut Commands,
    interactive_stuff: &mut InteractiveStuff,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    target: Entity,
) {
    // Check if the target entity exists and get its position for sound
    if let Ok((_, position, _, _)) = q_targets.get(target) {
        // Add a locked component with a 10-second timer
        let lock_timer = Timer::from_seconds(10.0, TimerMode::Once);
        commands.entity(target).insert(Locked(lock_timer));

        // Play door lock sound effect
        interactive_stuff.sound_events.write(SoundEvent {
            sound_file: "sounds/door_lock_heavy.ogg".to_string(),
            volume: 0.9,
            position: Some(*position),
        });
    } else {
        warn!(
            "GIS execution -> Lock interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}

/// Execute trip breaker interaction (turn off main power)
fn execute_trip_breaker_interaction(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    interactive_stuff: &mut InteractiveStuff,
    ev_bdr: &mut MessageWriter<BoardDataToRebuild>,
    q_targets: &Query<(
        &Behavior,
        &Position,
        Option<&Interactive>,
        Option<&RoomState>,
    )>,
    target: Entity,
) {
    // Get the behavior component to execute the interaction
    if let Ok((behavior, position, _interactive, room_state)) = q_targets.get(target) {
        // Execute the breaker trip using the existing InteractiveStuff system
        interactive_stuff.execute_interaction(
            target,
            position,
            None, // suppress default sound; custom breaker sound below
            behavior,
            room_state,
            InteractionExecutionType::ChangeState,
        );

        // Play breaker trip sound effect
        interactive_stuff.sound_events.write(SoundEvent {
            sound_file: "sounds/breaker_trip.ogg".to_string(),
            volume: 1.2,
            position: Some(*position),
        });

        // Spawn electrical sparks visual effect
        visual_effects::spawn_electrical_sparks(commands, asset_server, *position);

        // Rebuild lighting and collision data (this will turn off all lights)
        ev_bdr.write(BoardDataToRebuild {
            lighting: true,
            collision: true,
        });
    } else {
        warn!(
            "GIS execution -> TripBreaker interaction for {:?} FAILED: target entity not found or missing components",
            target
        );
    }
}
