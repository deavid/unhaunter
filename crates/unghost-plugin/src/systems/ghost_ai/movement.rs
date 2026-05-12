use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use rand::prelude::*;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use unboard_core::entity::GameSprite;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unboard_core::resources::roomdb::RoomTopology;
use uncommon_app_core::random_seed;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungearitems_core::components::salt::SaltyTrace;
use unghost_core::components::logic::ghost_death::{GhostDeathSequenceState, GhostDeathSignal};
use unghost_core::components::logic::ghost_influence::{GhostInfluence, InfluenceType};
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unghost_core::components::logic::red_light_charge::{GhostRedLightCharge, RedLightChargeMode};
use unghost_core::events::GhostAudioMessage;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;
use unlight_core::resources::light_grid::LightGrid;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::PlayerTag;
use unplayer_core::components::{Hiding, PlayerDisconnected, PlayerInactive, PlayerSpectating};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use unvitals_core::components::PlayerVitals;

use bevy_replicon::prelude::*;
use unreplicon_core::messages::SpawnParticleNetEvent;

use crate::metrics::GHOST_MOVEMENT;
use crate::systems::ghost_ai::roar::emit_ghost_audio;

// Constants for movement penalties
const WALL_AVOIDANCE_PENALTY: f32 = -100.0; // Negative because it's added to score
const FLOOR_CHANGE_PENALTY_BASE: f32 = -10.0; // Negative, base penalty for changing floors
const DISCHARGE_SPEED_MULTIPLIER: f32 = 1.45;
const CHARGING_SPEED_MULTIPLIER_IN_RED: f32 = 0.45;
const DISCHARGE_WARP_TRIGGER_CHANCE: i32 = 250;
const BASE_WARP_TRIGGER_CHANCE: i32 = 500;
const HUNT_DRAIN_BONUS_IN_RED: f32 = 0.65;
const RED_SEEK_RADIUS: i64 = 3;

#[derive(SystemParam)]
pub(crate) struct GhostDeathAudioParams<'w> {
    local_audio: LocalAudioEmitter<'w>,
    ev_audio: MessageWriter<'w, ToClients<GhostAudioMessage>>,
    local_player_role: Option<Res<'w, unreplicon_core::resources::LocalPlayerRole>>,
}

/// Updates the ghost's position based on its target location, hunting state, and
/// warping intensity.
///
/// This system handles the ghost's movement logic, ensuring it navigates the game
/// world according to its current state and objectives.
pub(crate) fn ghost_movement(
    mut q: Query<
        (
            &mut GhostSprite,
            &mut Position,
            Entity,
            Option<&mut GhostRedLightCharge>,
        ),
        (
            Without<PlayerTag>,
            Without<GhostInfluence>,
            Without<GhostDeathSignal>,
        ),
    >,
    qp: Query<
        (&Position, &PlayerVitals, Option<&Hiding>),
        (
            With<PlayerTag>,
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
    room_topology: Res<RoomTopology>,
    bf: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    mut commands: Commands,
    time: Res<Time>,
    config: Res<ObjectInteractionConfig>,
    object_query: Query<(&Position, &GhostInfluence)>,
    difficulty: Res<CurrentDifficulty>,
    light_grid: Option<Res<LightGrid>>,
    mut log_timer: Local<f32>,
    mut death_audio: GhostDeathAudioParams,
    mut particle_net_writer: MessageWriter<ToClients<SpawnParticleNetEvent>>,
    qp_breach: Query<&Position, Without<GhostSprite>>,
) {
    let measure = GHOST_MOVEMENT.time_measure();
    let has_local_player = death_audio.local_player_role.is_some();

    *log_timer -= time.delta_secs();
    if *log_timer <= 0.0 {
        *log_timer = 10.0;
        for (_, pos, entity, _) in q.iter() {
            info!("[ghost_movement] ghost {:?} position: {:?}", entity, pos);
        }
    }

    let mut rng = random_seed::rng();
    let dt = time.delta_secs() * 60.0;
    let dt_secs = time.delta_secs();
    let current_secs = time.elapsed_secs_f64();
    for (mut ghost, mut pos, entity, mut red_charge) in q.iter_mut() {
        let mut speed_multiplier = 1.0;
        let mut warp_trigger_chance = BASE_WARP_TRIGGER_CHANCE;
        let mut can_accumulate_warp = true;
        let mut force_red_seek_target = None;

        let old_z = pos.z.round();
        ghost.floor_stay_timer += dt_secs;

        if let Some(charge) = red_charge.as_deref_mut() {
            let red_intensity = sample_red_intensity_at(&light_grid, *pos, &bf);
            let in_reactive_red = red_intensity > charge.red_react_threshold;

            match charge.mode {
                RedLightChargeMode::Charging => {
                    if in_reactive_red {
                        charge.charge += red_intensity * charge.charge_rate * dt_secs;
                        can_accumulate_warp = false;
                        ghost.warp = 0.0;
                        speed_multiplier = CHARGING_SPEED_MULTIPLIER_IN_RED;
                        ghost.hunting -= HUNT_DRAIN_BONUS_IN_RED * dt_secs;
                        if ghost.hunting <= 0.0 {
                            ghost.hunting = 0.0;
                            if ghost.hunt_target {
                                info!("Hunt interrupted by sustained red-light exposure");
                            }
                            ghost.hunt_target = false;
                        }
                        force_red_seek_target = find_brightest_red_target(
                            &light_grid,
                            *pos,
                            &bf,
                            &board_collision,
                            RED_SEEK_RADIUS,
                        );
                    }

                    if charge.charge >= charge.threshold {
                        charge.mode = RedLightChargeMode::Discharging;
                        charge.charge = charge.threshold;
                        info!(
                            "Ghost {:?} reached red-light overload threshold; switching to discharge mode",
                            entity
                        );
                    }
                }
                RedLightChargeMode::Discharging => {
                    charge.charge -= charge.discharge_rate * dt_secs;
                    speed_multiplier = DISCHARGE_SPEED_MULTIPLIER;
                    warp_trigger_chance = DISCHARGE_WARP_TRIGGER_CHANCE;
                    if charge.charge <= 0.0 {
                        charge.charge = 0.0;
                        charge.mode = RedLightChargeMode::Charging;
                        info!(
                            "Ghost {:?} finished red-light discharge; returning to charging mode",
                            entity
                        );
                    }
                }
            }
        }

        if let Some(target_point) = ghost.target_point {
            let mut delta = target_point.delta(*pos);
            if can_accumulate_warp
                && rng.random_range(0..warp_trigger_chance) == 0
                && delta.distance() > 3.0
                && ghost.warp < 0.1
            {
                // Sometimes, warp ahead. This also is to increase visibility of the ghost
                ghost.warp += 40.0;
            }
            ghost.warp -= dt * 0.5;
            if ghost.warp < 0.0 {
                ghost.warp = 0.0;
            }
            // Expand the braking zone dynamically based on warp speed
            let braking_zone = 5.0 + (ghost.warp * 2.0);
            if delta.distance() < braking_zone {
                ghost.warp /= 1.2_f32.powf(dt);
            }
            let dlen = delta.distance() + 0.001;
            if dlen > 1.0 {
                delta.dx /= dlen.sqrt();
                delta.dy /= dlen.sqrt();
                delta.dz /= dlen.sqrt();
            }
            delta.dx *= ghost.warp + 1.0;
            delta.dy *= ghost.warp + 1.0;
            delta.dz *= ghost.warp + 1.0;
            let mut finalize = false;
            if ghost.hunt_target {
                if time.elapsed_secs() - ghost.hunt_time_secs > 1.0 {
                    if dlen < 4.0 {
                        delta.dx /= (dlen + 1.5) / 4.0;
                        delta.dy /= (dlen + 1.5) / 4.0;
                        delta.dz /= (dlen + 1.5) / 4.0;
                    }
                    pos.x += delta.dx / 70.0
                        * dt
                        * difficulty.0.ghost_hunting_aggression()
                        * speed_multiplier;
                    pos.y += delta.dy / 70.0
                        * dt
                        * difficulty.0.ghost_hunting_aggression()
                        * speed_multiplier;
                    pos.z += delta.dz / 10.0
                        * dt
                        * difficulty.0.ghost_hunting_aggression()
                        * speed_multiplier;
                    ghost.hunting -= dt / 60.0;
                }
                if ghost.hunting < 0.0 {
                    if ghost.hunt_target {
                        // Check if it was actually hunting
                        ghost.times_hunted_this_mission += 1;
                        // Removed mute event - mute is triggered anticipatory during hunt warning
                    }
                    ghost.hunting = 0.0;
                    ghost.hunt_target = false;
                    finalize = true;
                    info!("Hunt finished");
                }
            } else {
                pos.x += delta.dx / 200.0 * dt * difficulty.0.ghost_speed() * speed_multiplier;
                pos.y += delta.dy / 200.0 * dt * difficulty.0.ghost_speed() * speed_multiplier;
                pos.z += delta.dz / 20.0 * dt * difficulty.0.ghost_speed() * speed_multiplier;
            }
            pos.z = pos.z.clamp(0.0, (bf.map_size.2 - 1) as f32);
            if pos.z.round() != old_z {
                ghost.floor_stay_timer = 0.0;
            }
            if dlen < 0.5 {
                finalize = true;
            }
            if finalize {
                ghost.target_point = None;
            }
        }
        if let Some(seek_target) = force_red_seek_target {
            ghost.target_point = Some(seek_target);
        }
        if ghost.target_point.is_none() || (ghost.hunt_target && rng.random_range(0..60) == 0) {
            let mut target_point = ghost.spawn_point.to_position();
            let wander: f32 = rng.random_range(0.001..1.0_f32).powf(6.0) * 12.0 + 0.5;
            let dx: f32 = (0..5).map(|_| rng.random_range(-1.0..1.0)).sum();
            let dy: f32 = (0..5).map(|_| rng.random_range(-1.0..1.0)).sum();
            // Initial Z wandering: prefer staying on the same floor.
            let dz: f32 = (0..5)
                .map(|_| {
                    if rng.random_range(0.0..1.0) < 0.1 {
                        // 10% chance for a non-trivial dz component
                        rng.random_range(-0.1..0.1) // Small Z wander
                    } else {
                        0.0 // Most of the time, no Z wander component from this
                    }
                })
                .sum();
            let dist: f32 = (0..5).map(|_| rng.random_range(0.2..wander)).sum();
            let dd = ((dx * dx + dy * dy + dz * dz).sqrt() / dist.max(0.01)).max(0.01); // Include Z, ensure dd is not zero

            let mut hunt = false;
            target_point.x = (target_point.x + pos.x * wander) / (1.0 + wander) + dx / dd;
            target_point.y = (target_point.y + pos.y * wander) / (1.0 + wander) + dy / dd;
            target_point.z = (target_point.z + pos.z * wander) / (1.0 + wander) + dz / dd;
            target_point.z = target_point.z.round();
            let ghbonus = if ghost.hunt_target { 10000.0 } else { 0.0001 };
            if !ghost.hunt_warning_active
                && rng
                    .random_range(0.0..(ghost.hunting * 10.0 + ghbonus).sqrt().max(0.000001) * 10.0)
                    > 10.0
            {
                let player_pos_l: Vec<(&Position, bool)> = qp
                    .iter()
                    .filter(|(_, v, _)| v.health > 0.0)
                    .map(
                        |(pos, _, hiding): (&Position, &PlayerVitals, Option<&Hiding>)| {
                            (pos, hiding.is_some())
                        },
                    )
                    .collect();
                if !player_pos_l.is_empty() {
                    let idx = rng.random_range(0..player_pos_l.len());
                    let (ppos, hiding) = player_pos_l[idx];

                    let search_radius = if hiding { 2.0 } else { 1.0 };
                    let mut old_target = ghost.target_point.unwrap_or(*pos);

                    // --- THE CORRECTED FIX ---
                    let mut abort_hunt = false;
                    if ghost.hunt_target {
                        // Measure how far the TARGET moved since 1 second ago.
                        // A player can only run so far. If it's > 10 tiles, it swapped to Player B!
                        // We ONLY penalize if we already had a target point (established chase),
                        // not during initial acquisition.
                        if let Some(actual_old_target) = ghost.target_point {
                            let target_jump_dist = actual_old_target.distance(ppos);

                            if target_jump_dist > 10.0 {
                                // Only penalize the massive map-crossing jump
                                let rage_penalty = target_jump_dist * 0.5;
                                ghost.rage = (ghost.rage - rage_penalty).max(0.0);

                                if ghost.rage < ghost.rage_limit {
                                    abort_hunt = true;
                                    ghost.hunting = 0.0;
                                    info!(
                                        "Ghost dropped hunt: target swapped/jumped ({:.1} tiles), rage drained.",
                                        target_jump_dist
                                    );
                                }
                            }
                        }
                    }
                    // -------------------------

                    old_target.x += rng.random_range(-search_radius..search_radius);
                    old_target.y += rng.random_range(-search_radius..search_radius);
                    old_target.z += rng.random_range(-search_radius / 2.0..search_radius / 2.0); // Add small Z randomization
                    let ppos = if hiding || ghost.calm_time_secs > 5.0 {
                        old_target
                    } else {
                        *ppos
                    };
                    ghost.calm_time_secs -= 2.0_f32.min(ghost.calm_time_secs);
                    let mut rng = random_seed::rng();
                    let random_offset = Vec2::new(
                        rng.random_range(-search_radius..search_radius),
                        rng.random_range(-search_radius..search_radius),
                    );
                    target_point.x = ppos.x + random_offset.x;
                    target_point.y = ppos.y + random_offset.y;
                    target_point.z = ppos.z.round();
                    hunt = !abort_hunt; // If we aborted, hunt is safely false!
                }
            }

            // --- Sample Potential Destinations and Calculate Scores ---
            if !hunt {
                let mut potential_destinations: Vec<(f32, Position)> = Vec::new();

                for _ in 0..config.num_destination_points_to_sample {
                    let mut candidate_dest = ghost.spawn_point.to_position(); // Base for wandering
                    let wander: f32 = rng.random_range(0.001..1.0_f32).powf(6.0) * 12.0 + 0.5;
                    let dx: f32 = (0..5).map(|_| rng.random_range(-1.0..1.0)).sum();
                    let dy: f32 = (0..5).map(|_| rng.random_range(-1.0..1.0)).sum();
                    let dz: f32 = (0..5).map(|_| rng.random_range(-0.5..0.5)).sum(); // Allow Z exploration for samples
                    let dist_norm_factor: f32 = (0..5).map(|_| rng.random_range(0.2..wander)).sum();
                    let dd_sample = ((dx * dx + dy * dy + dz * dz).sqrt()
                        / dist_norm_factor.max(0.01))
                    .max(0.01);

                    candidate_dest.x =
                        (candidate_dest.x + pos.x * wander) / (1.0 + wander) + dx / dd_sample;
                    candidate_dest.y =
                        (candidate_dest.y + pos.y * wander) / (1.0 + wander) + dy / dd_sample;
                    candidate_dest.z =
                        (candidate_dest.z + pos.z * wander) / (1.0 + wander) + dz / dd_sample;
                    candidate_dest.z = candidate_dest.z.round(); // Snap to floor

                    // Clamp candidate destination to map bounds before scoring
                    candidate_dest.x = candidate_dest.x.clamp(0.0, (bf.map_size.0 - 1) as f32);
                    candidate_dest.y = candidate_dest.y.clamp(0.0, (bf.map_size.1 - 1) as f32);
                    candidate_dest.z = candidate_dest.z.clamp(0.0, (bf.map_size.2 - 1) as f32);

                    let mut score = 1.0; // Base score
                    score +=
                        calculate_object_influence_score(candidate_dest, &object_query, &config)
                            / difficulty.0.ghost_attraction_to_breach().max(0.1); // Scale object influence
                    let penalty = 1.0
                        + calculate_movement_penalties(
                            candidate_dest,
                            &pos,
                            &ghost,
                            &bf,
                            &board_collision,
                            &difficulty,
                        )
                        .abs()
                            / 10.0;
                    score /= penalty;
                    potential_destinations.push((score, candidate_dest));
                }

                // --- Select Destination with Highest Score ---
                let mut best_destination = ghost.spawn_point.to_position();
                best_destination.z = pos.z.round().clamp(0.0, (bf.map_size.2 - 1) as f32); // Default to current floor

                let mut best_score = f32::MIN;

                for (score, point) in potential_destinations {
                    if score > best_score {
                        let point_bpos = point.to_board_position();
                        if point_bpos.is_valid(bf.map_size)
                            && board_collision.0[point_bpos.ndidx()].player_free
                        {
                            best_score = score;
                            best_destination = point;
                        }
                    }
                }
                target_point = best_destination;
            }
            // Clamp final target_point to map bounds (important if not from sampling or if sampling failed)
            target_point.x = target_point.x.clamp(0.0, (bf.map_size.0 - 1) as f32);
            target_point.y = target_point.y.clamp(0.0, (bf.map_size.1 - 1) as f32);
            target_point.z = target_point.z.clamp(0.0, (bf.map_size.2 - 1) as f32);
            let bpos = target_point.to_board_position();
            let dstroom = room_topology.room_tiles.get(&bpos);
            if dstroom.is_some() && board_collision.0[bpos.ndidx()].ghost_free {
                if hunt {
                    if !ghost.hunt_target {
                        ghost.hunt_time_secs = time.elapsed_secs();
                        info!("Hunting player for {:.1}s", ghost.hunting);
                        // Removed mute event - now triggered anticipatory during hunt warning
                    }
                } else if ghost.hunt_target {
                    info!("Hunt temporarily ended (remaining) {:.1}s", ghost.hunting);
                }
                // Final check to ensure the chosen bpos is valid before assigning.
                // This is somewhat redundant with checks in sampling, but good for safety.
                if bpos.is_valid(bf.map_size) && board_collision.0[bpos.ndidx()].ghost_free {
                    ghost.target_point = Some(target_point);
                }
                ghost.hunt_target = hunt;
            } else if ghost
                .target_point
                .map(|gp| pos.distance(&gp))
                .unwrap_or_default()
                < 0.5
            {
                ghost.hunt_target = false;
            }
        }
        if ghost.get_health() < 0.0 {
            // 1. Emit smoke effect over the network
            particle_net_writer.write(ToClients {
                mode: SendMode::Broadcast,
                message: SpawnParticleNetEvent {
                    particle_type: "smoke".to_string(),
                    position: [pos.x, pos.y, pos.z],
                },
            });

            // 2. Play the opening death roar through the explicit ghost audio path.
            emit_ghost_audio(
                "sounds/ghost-roar-1.ogg".to_string(),
                2.0,
                *pos,
                &mut death_audio.local_audio,
                &mut death_audio.ev_audio,
                has_local_player,
            );

            if let Some(breach) = ghost.breach_id {
                commands
                    .entity(breach)
                    .insert(GhostDeathSignal::new(current_secs, 5.0));

                if let Ok(breach_pos) = qp_breach.get(breach) {
                    particle_net_writer.write(ToClients {
                        mode: SendMode::Broadcast,
                        message: SpawnParticleNetEvent {
                            particle_type: "smoke".to_string(),
                            position: [breach_pos.x, breach_pos.y, breach_pos.z],
                        },
                    });
                }
            }
            commands.entity(entity).insert((
                GhostDeathSignal::new(current_secs, 5.0),
                GhostDeathSequenceState::default(),
            ));
        }
    }
    measure.end_ms();
}

fn sample_red_intensity_at(
    light_grid: &Option<Res<LightGrid>>,
    pos: Position,
    bf: &Res<BoardTopology>,
) -> f32 {
    let Some(light_grid) = light_grid else {
        return 0.0;
    };
    let bpos = pos.to_board_position();
    if !bpos.is_valid(bf.map_size) {
        return 0.0;
    }
    light_grid
        .light_field
        .get(bpos.ndidx())
        .map(|f| f.additional.red.max(0.0))
        .unwrap_or(0.0)
}

fn find_brightest_red_target(
    light_grid: &Option<Res<LightGrid>>,
    pos: Position,
    bf: &Res<BoardTopology>,
    board_collision: &Res<BoardCollisionField>,
    radius: i64,
) -> Option<Position> {
    let Some(light_grid) = light_grid else {
        return None;
    };

    let origin = pos.to_board_position();
    if !origin.is_valid(bf.map_size) {
        return None;
    }

    let mut best = origin.clone();
    let mut best_red = light_grid
        .light_field
        .get(origin.ndidx())
        .map(|f| f.additional.red)
        .unwrap_or(0.0);

    for dz in -1..=1 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let candidate = BoardPosition {
                    x: origin.x + dx,
                    y: origin.y + dy,
                    z: origin.z + dz,
                };
                if !candidate.is_valid(bf.map_size) {
                    continue;
                }
                let idx = candidate.ndidx();
                if !board_collision.0[idx].ghost_free {
                    continue;
                }
                let red = light_grid
                    .light_field
                    .get(idx)
                    .map(|f| f.additional.red)
                    .unwrap_or(0.0);
                if red > best_red {
                    best_red = red;
                    best = candidate;
                }
            }
        }
    }

    if best == origin {
        None
    } else {
        Some(best.to_position())
    }
}

/// Calculates the score contribution from object influences.
fn calculate_object_influence_score(
    potential_destination: Position,
    object_query: &Query<(&Position, &GhostInfluence)>,
    config: &Res<ObjectInteractionConfig>,
) -> f32 {
    let mut score = 0.0;
    // Iterate through objects with GhostInfluence
    for (object_position, ghost_influence) in object_query.iter() {
        let distance2 = potential_destination.distance2_zf(object_position, 12.0);
        let dist = distance2.sqrt();

        // Apply influence based on distance and charge value
        match ghost_influence.influence_type {
            InfluenceType::Attractive => {
                // Add to score for Attractive objects, weighted by attractive_influence_multiplier
                // Use 1/(d+5) for a slower falloff at distance
                score += config.attractive_influence_multiplier * ghost_influence.charge_value
                    / (dist + 5.0);
            }
            InfluenceType::Repulsive => {
                // Subtract from score for Repulsive objects, weighted by
                // repulsive_influence_multiplier
                score -= config.repulsive_influence_multiplier * ghost_influence.charge_value
                    / (dist + 5.0);
            }
        }
    }
    score
}

/// Calculates penalties for movement choices (walls, floor changes).
fn calculate_movement_penalties(
    potential_destination: Position,
    current_ghost_pos: &Position,
    current_ghost_sprite: &GhostSprite,
    bf: &Res<BoardTopology>,
    board_collision: &Res<BoardCollisionField>,
    _difficulty: &Res<CurrentDifficulty>, // Available for future use if penalties scale with difficulty
) -> f32 {
    let mut penalty_score = 0.0;
    let dest_bpos = potential_destination.to_board_position();

    // Bounds check (should be redundant if candidate generation is correct, but good for safety)
    if !dest_bpos.is_valid(bf.map_size) {
        return f32::MIN / 2.0; // Heavily penalize out-of-bounds
    }

    // Wall Avoidance Penalty
    // Penalize if the destination tile itself is not player_free (we don't use ghost_free here because that would be for future use on pathfinding)
    if !board_collision.0[dest_bpos.ndidx()].player_free {
        penalty_score += WALL_AVOIDANCE_PENALTY;
    }

    // Floor Change Penalty
    // Penalize if the destination is on a different floor (rounded Z)
    if potential_destination.z.round() != current_ghost_pos.z.round() {
        // Only allow floor change if we have been on this floor for at least 8 seconds.
        // During a hunt, we allow floor changes regardless of time to prevent kiting.
        let mut floor_change_penalty = FLOOR_CHANGE_PENALTY_BASE;
        if current_ghost_sprite.floor_stay_timer < 8.0 && !current_ghost_sprite.hunt_target {
            floor_change_penalty *= 20.0; // Extremely heavy penalty to prevent rapid floor switching
        }
        penalty_score += floor_change_penalty;
    }

    penalty_score
}

/// Spawns a shared salty-trace skeleton at the given tile.
pub(crate) fn spawn_salty_trace(commands: &mut Commands, tile_position: BoardPosition) {
    commands.spawn((
        SaltyTrace,
        tile_position.to_position(),
        Replicated,
        GameSprite,
    ));
}
