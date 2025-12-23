use bevy::prelude::*;
use rand::Rng;
use uncore_board::behavior::Behavior;
use uncore_board::behavior::TileState;
use uncore_board::behavior::component::{Door, InteractableByGhost};
use uncore_board::resources::board_data::BoardData;
use uncore_events::events::ghost_interaction::{GhostInteractionEvent, GhostInteractionType};
use uncore_foundation::random_seed;
use undifficulty::CurrentDifficulty;
use unghost_core::components::GhostSprite;
use unrender::VisibilityData;
use unspatial::Position;
use untags::PlayerTag;

use crate::components::interaction::Locked;

// Simple debug toggle to make GIS interactions more frequent and verbose during development
const GIS_DEBUG: bool = false;

/// Registers selection systems with the Bevy app
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(bevy::prelude::Update, ghost_interaction_selection_system);
}

/// System that determines when and what interactions a ghost performs based on personality.
///
/// This system replaces the old distance-based probability system with a personality-driven
/// approach that considers ghost rage state, player visibility, and object-specific rules.
fn ghost_interaction_selection_system(
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    board_data: Res<BoardData>,
    visibility_data: Res<VisibilityData>,
    q_player: Query<&Position, With<PlayerTag>>,
    q_ghost: Query<(&GhostSprite, &Position)>,
    q_interactables: Query<(
        Entity,
        &Position,
        &Behavior,
        Option<&Door>,
        Option<&Locked>,
        Option<&InteractableByGhost>,
    )>,
    mut ev_ghost_interaction: MessageWriter<GhostInteractionEvent>,
) {
    let mut rng = random_seed::rng();

    // Find the ghost
    let Ok((ghost_sprite, ghost_pos)) = q_ghost.single() else {
        return;
    };

    // Get ghost personality for this ghost type
    let personality = ghost_sprite.class.personality();

    // Calculate current rage ratio (0.0 = calm, 1.0 = max rage)
    let rage_ratio = (ghost_sprite.rage / ghost_sprite.rage_limit).clamp(0.0, 1.0);

    // Apply difficulty multiplier to interaction frequency
    let difficulty_multiplier = difficulty.0.ghost_interaction_frequency;

    // Check each interaction type for probability
    let interaction_types = [
        (GhostInteractionType::Toggle, personality.toggle_rate),
        (GhostInteractionType::DoorSlam, personality.door_slam_rate),
        (GhostInteractionType::DoorCreak, personality.door_creak_rate),
        (GhostInteractionType::Throw, personality.throw_rate),
        (GhostInteractionType::Nudge, personality.nudge_rate),
        (
            GhostInteractionType::HauntedMove,
            personality.haunted_move_rate,
        ),
        (GhostInteractionType::Lock, personality.lock_rate),
        (
            GhostInteractionType::TripBreaker,
            personality.trip_breaker_rate,
        ),
    ];

    for (interaction_type, (calm_rate, angry_rate)) in interaction_types {
        // Interpolate rate based on rage level
        let current_rate = calm_rate + (angry_rate - calm_rate) * rage_ratio;

        // Apply difficulty multiplier
        let adjusted_rate = current_rate * difficulty_multiplier;

        // Convert from per-hour to per-frame probability
        let mut chance_this_frame = (adjusted_rate / 3600.0) * time.delta().as_secs_f32();

        // When debugging, scale up the frequency aggressively to observe interactions
        if GIS_DEBUG {
            // Baseline: ~60x increases converts per-hour to per-minute behavior while running
            chance_this_frame *= 60.0;
        }

        // Roll for this interaction type
        if rng.random_range(0.0..1.0) < chance_this_frame {
            // Try to find a suitable target for this interaction
            if let Some((target, destination)) = find_interaction_target(
                interaction_type,
                ghost_pos,
                &q_interactables,
                &q_player,
                &board_data,
                &visibility_data,
                &mut rng,
            ) {
                // Dispatch the interaction event
                ev_ghost_interaction.write(GhostInteractionEvent {
                    target,
                    interaction_type,
                    destination,
                });

                if GIS_DEBUG {
                    // One-line log for emitted interaction
                    if let Some(p) = destination {
                        info!(
                            "GIS selection -> emitted {:?} to {:?} with dest ({:.2}, {:.2}, {:.2})",
                            interaction_type, target, p.x, p.y, p.z
                        );
                    } else {
                        info!(
                            "GIS selection -> emitted {:?} to {:?}",
                            interaction_type, target
                        );
                    }
                }

                // Only trigger one interaction per frame to avoid spam
                return;
            }
        }
    }
}

/// Finds a suitable target entity for the given interaction type.
///
/// Returns (target_entity, optional_destination) if a valid target is found.
fn find_interaction_target(
    interaction_type: GhostInteractionType,
    ghost_pos: &Position,
    q_interactables: &Query<(
        Entity,
        &Position,
        &Behavior,
        Option<&Door>,
        Option<&Locked>,
        Option<&InteractableByGhost>,
    )>,
    q_player: &Query<&Position, With<PlayerTag>>,
    board_data: &BoardData,
    visibility_data: &VisibilityData,
    rng: &mut impl Rng,
) -> Option<(Entity, Option<Position>)> {
    // Respect production interaction radius at all times (do not increase in debug)
    const MAX_INTERACTION_DISTANCE: f32 = 4.0;
    const MAX_INTERACTION_DISTANCE_SQ: f32 = MAX_INTERACTION_DISTANCE * MAX_INTERACTION_DISTANCE;

    // Find all entities within interaction range
    let nearby_entities: Vec<_> = q_interactables
        .iter()
        .filter(|(_, pos, _, _, _, _)| ghost_pos.distance2(pos) <= MAX_INTERACTION_DISTANCE_SQ)
        .collect();

    if nearby_entities.is_empty() {
        return None;
    }

    let nearby_count = nearby_entities.len();
    // Debug counters for why interactions may fail to find targets
    let mut can_emit_light_count = 0usize;
    let mut doors_total = 0usize;
    let mut doors_open = 0usize;
    let mut doors_closed = 0usize;
    let mut doors_locked = 0usize;
    let mut doors_unlocked = 0usize;
    let mut breaker_on = 0usize;
    let mut breaker_off = 0usize;
    let mut throwable_flag = 0usize;
    let mut throwable_with_dest = 0usize;
    let mut nudgeable_flag = 0usize;
    let mut haunt_flag = 0usize;
    let mut haunt_with_dest = 0usize;

    // Filter entities based on interaction-specific rules
    let suitable_targets: Vec<_> = nearby_entities
        .into_iter()
        .filter_map(
            |(entity, pos, behavior, door_comp, locked_comp, ghost_marker)| {
                // In production, require the InteractableByGhost marker to match existing behavior
                if !GIS_DEBUG && ghost_marker.is_none() {
                    return None;
                }
                match interaction_type {
                    GhostInteractionType::Toggle => {
                        // Can toggle lights, lamps, or switches.
                        let class = behavior.class();
                        let is_switch = matches!(
                            class,
                            uncore_board::behavior::Class::Switch
                                | uncore_board::behavior::Class::RoomSwitch
                        );
                        let is_lamp = matches!(
                            class,
                            uncore_board::behavior::Class::WallLamp
                                | uncore_board::behavior::Class::FloorLamp
                                | uncore_board::behavior::Class::TableLamp
                                | uncore_board::behavior::Class::CeilingLight
                                | uncore_board::behavior::Class::StreetLight
                                | uncore_board::behavior::Class::CandleLight
                        );
                        if behavior.can_emit_light()
                            || behavior.p.light.can_emit_light
                            || is_switch
                            || is_lamp
                        {
                            if GIS_DEBUG {
                                can_emit_light_count += 1;
                            }
                            Some((entity, pos, None))
                        } else {
                            None
                        }
                    }
                    GhostInteractionType::DoorSlam | GhostInteractionType::DoorCreak => {
                        // Only target actual doors that are currently open
                        if GIS_DEBUG && door_comp.is_some() {
                            doors_total += 1;
                            match behavior.state() {
                                TileState::Open => doors_open += 1,
                                TileState::Closed => doors_closed += 1,
                                _ => {}
                            }
                        }
                        // For DoorCreak allow both open and closed doors (creak while opening/closing)
                        if door_comp.is_some()
                            && (interaction_type == GhostInteractionType::DoorCreak
                                || behavior.state() == TileState::Open)
                        {
                            Some((entity, pos, None))
                        } else {
                            None
                        }
                    }
                    GhostInteractionType::Lock => {
                        // Only target doors that are closed and not already locked
                        if GIS_DEBUG && door_comp.is_some() {
                            doors_total += 1;
                            if locked_comp.is_some() {
                                doors_locked += 1;
                            } else {
                                doors_unlocked += 1;
                            }
                        }
                        if door_comp.is_some()
                            && locked_comp.is_none()
                            && behavior.state() == TileState::Closed
                        {
                            Some((entity, pos, None))
                        } else {
                            None
                        }
                    }
                    GhostInteractionType::TripBreaker => {
                        // Only target breakers that are currently On
                        if GIS_DEBUG && behavior.class() == uncore_board::behavior::Class::Breaker {
                            if behavior.state() == TileState::On {
                                breaker_on += 1;
                            } else {
                                breaker_off += 1;
                            }
                        }
                        if behavior.class() == uncore_board::behavior::Class::Breaker
                            && behavior.state() == TileState::On
                        {
                            Some((entity, pos, None))
                        } else {
                            None
                        }
                    }
                    GhostInteractionType::Throw => {
                        // Only items explicitly throwable or movable are eligible
                        if behavior.p.object.throwable || behavior.p.object.movable {
                            if GIS_DEBUG {
                                throwable_flag += 1;
                            }
                            // Find a destination for throwing with collision checking
                            if let Some(destination) =
                                find_throw_destination(entity, pos, q_player, board_data, rng)
                            {
                                if GIS_DEBUG {
                                    throwable_with_dest += 1;
                                }
                                Some((entity, pos, Some(destination)))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    GhostInteractionType::Nudge => {
                        // Only items explicitly nudgeable or movable are eligible
                        if behavior.p.object.nudgeable || behavior.p.object.movable {
                            if GIS_DEBUG {
                                nudgeable_flag += 1;
                            }
                            // Find a small, nearby floor destination to nudge towards.
                            // Only emit if a safe destination is found to avoid collisions/overlaps.
                            if let Some(destination) =
                                find_nudge_destination(entity, pos, board_data, rng)
                            {
                                Some((entity, pos, Some(destination)))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    GhostInteractionType::HauntedMove => {
                        // Only items explicitly haunt_movable or movable are eligible
                        if behavior.p.object.haunt_movable || behavior.p.object.movable {
                            if GIS_DEBUG {
                                haunt_flag += 1;
                            }
                            // Find a destination for haunted movement with collision checking
                            if let Some(destination) =
                                find_movement_destination(entity, pos, board_data, rng)
                            {
                                if GIS_DEBUG {
                                    haunt_with_dest += 1;
                                }
                                Some((entity, pos, Some(destination)))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
            },
        )
        .collect();

    if suitable_targets.is_empty() {
        if GIS_DEBUG {
            match interaction_type {
                GhostInteractionType::Toggle => {
                    info!(
                        "GIS selection -> no target for Toggle: nearby={}, togglables(lights+switches)={}",
                        nearby_count, can_emit_light_count
                    );
                }
                GhostInteractionType::DoorSlam | GhostInteractionType::DoorCreak => {
                    info!(
                        "GIS selection -> no door target: nearby={}, doors_total={}, open={}, closed={}",
                        nearby_count, doors_total, doors_open, doors_closed
                    );
                }
                GhostInteractionType::Lock => {
                    info!(
                        "GIS selection -> no lock target: nearby={}, doors_total={}, closed_unlocked={}, locked={}",
                        nearby_count, doors_total, doors_unlocked, doors_locked
                    );
                }
                GhostInteractionType::TripBreaker => {
                    info!(
                        "GIS selection -> no breaker target: nearby={}, breakers_on={}, breakers_off={}",
                        nearby_count, breaker_on, breaker_off
                    );
                }
                GhostInteractionType::Throw => {
                    info!(
                        "GIS selection -> no throw target: nearby={}, throwable_flag={}, with_dest={}",
                        nearby_count, throwable_flag, throwable_with_dest
                    );
                }
                GhostInteractionType::Nudge => {
                    info!(
                        "GIS selection -> no nudge target: nearby={}, nudgeable_flag={}",
                        nearby_count, nudgeable_flag
                    );
                }
                GhostInteractionType::HauntedMove => {
                    info!(
                        "GIS selection -> no haunted move target: nearby={}, haunt_flag={}, with_dest={}",
                        nearby_count, haunt_flag, haunt_with_dest
                    );
                }
            }
        }
        return None;
    }

    // Prioritize targets based on player visibility for dramatic effect
    let _player_pos = q_player.single().ok()?;

    // Separate targets into visible and non-visible to player
    let mut visible_targets = Vec::new();
    let mut hidden_targets = Vec::new();

    for (entity, pos, destination) in suitable_targets {
        let target_board_pos = pos.to_board_position();

        // Check if the target is visible to the player using the visibility data
        let is_visible_to_player =
            if let Some(idx) = target_board_pos.ndidx_checked(board_data.map_size) {
                visibility_data
                    .visibility_field
                    .get(idx)
                    .copied()
                    .unwrap_or(0.0)
                    > 0.1
            } else {
                false
            };

        if is_visible_to_player {
            visible_targets.push((entity, destination));
        } else {
            hidden_targets.push((entity, destination));
        }
    }

    // Prefer visible targets for dramatic effect (70% chance), otherwise use hidden targets
    let use_visible = !visible_targets.is_empty() && rng.random_range(0.0..1.0) < 0.7;

    let chosen_targets = if use_visible && !visible_targets.is_empty() {
        &visible_targets
    } else if !hidden_targets.is_empty() {
        &hidden_targets
    } else {
        &visible_targets // Fallback to visible if hidden is empty
    };

    if chosen_targets.is_empty() {
        return None;
    }

    // Select a random target from the chosen category
    let (entity, destination) = chosen_targets[rng.random_range(0..chosen_targets.len())];
    Some((entity, destination))
}

/// Finds a suitable destination for throwing an object.
///
/// Prefers locations that are visible to the player for maximum impact.
fn find_throw_destination(
    _entity: Entity,
    object_pos: &Position,
    q_player: &Query<&Position, With<PlayerTag>>,
    board_data: &BoardData,
    rng: &mut impl Rng,
) -> Option<Position> {
    // Get player position for dramatic effect preference
    let player_pos = q_player.single().ok()?;

    // Try to find a valid destination near the player for maximum impact
    for _ in 0..10 {
        let offset_x = rng.random_range(-3.0..3.0);
        let offset_y = rng.random_range(-3.0..3.0);

        let candidate_pos = Position {
            x: player_pos.x + offset_x,
            y: player_pos.y + offset_y,
            z: player_pos
                .z
                .floor()
                .clamp(0.0, (board_data.map_size.2 as f32) - 1.0), // Clamp to valid floor range
            global_z: 0.0,
        };

        // Check if destination is valid (walkable floor) AND within map bounds
        let board_pos = candidate_pos.to_board_position();
        if let Some(idx) = board_pos.ndidx_checked(board_data.map_size)
            && let Some(collision_data) = board_data.collision_field.get(idx)
            && collision_data.player_free
        {
            return Some(candidate_pos);
        }
    }

    // If no good position near player found, try closer to object
    for _ in 0..5 {
        let offset_x = rng.random_range(-2.0..2.0);
        let offset_y = rng.random_range(-2.0..2.0);

        let candidate_pos = Position {
            x: object_pos.x + offset_x,
            y: object_pos.y + offset_y,
            z: object_pos
                .z
                .floor()
                .clamp(0.0, (board_data.map_size.2 as f32) - 1.0), // Clamp to valid floor range
            global_z: 0.0,
        };

        let board_pos = candidate_pos.to_board_position();
        if let Some(idx) = board_pos.ndidx_checked(board_data.map_size)
            && let Some(collision_data) = board_data.collision_field.get(idx)
            && collision_data.player_free
        {
            return Some(candidate_pos);
        }
    }

    None
}

/// Finds a suitable destination for haunted object movement.
///
/// Selects a nearby location for slow, eerie sliding movement.
fn find_movement_destination(
    _entity: Entity,
    object_pos: &Position,
    board_data: &BoardData,
    rng: &mut impl Rng,
) -> Option<Position> {
    // Try several nearby positions for subtle movement
    for _ in 0..8 {
        let offset_x = rng.random_range(-2.0..2.0);
        let offset_y = rng.random_range(-2.0..2.0);

        let candidate_pos = Position {
            x: object_pos.x + offset_x,
            y: object_pos.y + offset_y,
            z: object_pos
                .z
                .floor()
                .clamp(0.0, (board_data.map_size.2 as f32) - 1.0), // Clamp to valid floor range
            global_z: object_pos.global_z,
        };

        // Check if destination is valid (walkable floor) AND within map bounds
        let board_pos = candidate_pos.to_board_position();
        if let Some(idx) = board_pos.ndidx_checked(board_data.map_size)
            && let Some(collision_data) = board_data.collision_field.get(idx)
            && collision_data.player_free
        {
            return Some(candidate_pos);
        }
    }

    None
}

/// Finds a small nearby floor destination for a nudge.
fn find_nudge_destination(
    _entity: Entity,
    object_pos: &Position,
    board_data: &BoardData,
    rng: &mut impl Rng,
) -> Option<Position> {
    // Try a handful of tiny offsets around the object
    for _ in 0..8 {
        let offset_x = rng.random_range(-1.0..1.0);
        let offset_y = rng.random_range(-1.0..1.0);

        let candidate_pos = Position {
            x: object_pos.x + offset_x,
            y: object_pos.y + offset_y,
            z: object_pos
                .z
                .floor()
                .clamp(0.0, (board_data.map_size.2 as f32) - 1.0), // Clamp to valid floor range
            global_z: object_pos.global_z,
        };

        let board_pos = candidate_pos.to_board_position();
        if let Some(idx) = board_pos.ndidx_checked(board_data.map_size)
            && let Some(collision_data) = board_data.collision_field.get(idx)
            && collision_data.player_free
        {
            return Some(candidate_pos);
        }
    }

    None
}
