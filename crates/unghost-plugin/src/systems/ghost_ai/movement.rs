use bevy::color::palettes::css;
use bevy::prelude::*;
use rand::Rng;
use unbehavior::roomdb::RoomDB;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::random_seed;
use ungearitems_core::components::salt::{SaltyTrace, SaltyTraceTimer, UVReactive};
use unghost_core::components::ghost_influence::{GhostInfluence, InfluenceType};
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::{Hiding, PlayerSprite};
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::PlayerTag;

use crate::components::fade_out::FadeOut;
use crate::metrics::GHOST_MOVEMENT;

// Constants for movement penalties
const WALL_AVOIDANCE_PENALTY: f32 = -100.0; // Negative because it's added to score
const FLOOR_CHANGE_PENALTY_BASE: f32 = -50.0; // Negative, base penalty for changing floors

/// Updates the ghost's position based on its target location, hunting state, and
/// warping intensity.
///
/// This system handles the ghost's movement logic, ensuring it navigates the game
/// world according to its current state and objectives.
pub(crate) fn ghost_movement(
    mut q: Query<
        (&mut GhostSprite, &mut Position, Entity),
        (
            Without<PlayerTag>,
            Without<GhostInfluence>,
            Without<FadeOut>,
        ),
    >,
    qp: Query<(&Position, &PlayerSprite, Option<&Hiding>), With<PlayerTag>>,
    roomdb: Res<RoomDB>,
    mut summary: ResMut<SummaryData>,
    bf: Res<BoardTopology>,
    board_collision: Res<BoardCollisionField>,
    mut commands: Commands,
    time: Res<Time>,
    config: Res<ObjectInteractionConfig>,
    object_query: Query<(&Position, &GhostInfluence)>,
    difficulty: Res<CurrentDifficulty>,
) {
    let measure = GHOST_MOVEMENT.time_measure();

    let mut rng = random_seed::rng();
    let dt = time.delta_secs() * 60.0;
    for (mut ghost, mut pos, entity) in q.iter_mut() {
        if let Some(target_point) = ghost.target_point {
            let mut delta = target_point.delta(*pos);
            if rng.random_range(0..500) == 0 && delta.distance() > 3.0 && ghost.warp < 0.1 {
                // Sometimes, warp ahead. This also is to increase visibility of the ghost
                ghost.warp += 40.0;
            }
            ghost.warp -= dt * 0.5;
            if ghost.warp < 0.0 {
                ghost.warp = 0.0;
            }
            if delta.distance() < 5.0 {
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
                    pos.x += delta.dx / 70.0 * dt * difficulty.0.ghost_hunting_aggression;
                    pos.y += delta.dy / 70.0 * dt * difficulty.0.ghost_hunting_aggression;
                    pos.z += delta.dz / 10.0 * dt * difficulty.0.ghost_hunting_aggression;
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
                    warn!("Hunt finished");
                }
            } else {
                pos.x += delta.dx / 200.0 * dt * difficulty.0.ghost_speed;
                pos.y += delta.dy / 200.0 * dt * difficulty.0.ghost_speed;
                pos.z += delta.dz / 20.0 * dt * difficulty.0.ghost_speed;
            }
            pos.z = pos.z.clamp(0.0, (bf.map_size.2 - 1) as f32);
            if dlen < 0.5 {
                finalize = true;
            }
            if finalize {
                ghost.target_point = None;
            }
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
                    .filter(|(_, p, _)| p.health > 0.0)
                    .map(|(pos, _, hiding)| (pos, hiding.is_some()))
                    .collect();
                if !player_pos_l.is_empty() {
                    let idx = rng.random_range(0..player_pos_l.len());
                    let (ppos, hiding) = player_pos_l[idx];
                    let search_radius = if hiding { 2.0 } else { 1.0 };
                    let mut old_target = ghost.target_point.unwrap_or(*pos);
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
                    hunt = true;
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
                            / difficulty.0.ghost_attraction_to_breach.max(0.1); // Scale object influence
                    let penalty = 1.0
                        + calculate_movement_penalties(
                            candidate_dest,
                            &pos,
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
            let dstroom = roomdb.room_tiles.get(&bpos);
            if dstroom.is_some() && board_collision.0[bpos.ndidx()].ghost_free {
                if hunt {
                    if !ghost.hunt_target {
                        ghost.hunt_time_secs = time.elapsed_secs();
                        warn!("Hunting player for {:.1}s", ghost.hunting);
                        // Removed mute event - now triggered anticipatory during hunt warning
                    }
                } else if ghost.hunt_target {
                    warn!("Hunt temporarily ended (remaining) {:.1}s", ghost.hunting);
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
            summary.ghosts_unhaunted += 1;
            if let Some(breach) = ghost.breach_id {
                commands
                    .entity(breach)
                    .insert(FadeOut::new(5.0))
                    .insert(MapColor {
                        color: Color::WHITE.with_alpha(1.0),
                    });
            }
            commands
                .entity(entity)
                .insert(FadeOut::new(5.0))
                .insert(MapColor {
                    color: Color::WHITE.with_alpha(1.0),
                });
        }
    }
    measure.end_ms();
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
        let distance2 = potential_destination.distance2_zf(object_position, 20.0);

        // Apply influence based on distance and charge value
        match ghost_influence.influence_type {
            InfluenceType::Attractive => {
                // Add to score for Attractive objects, weighted by attractive_influence_multiplier
                score += config.attractive_influence_multiplier * ghost_influence.charge_value
                    / (distance2 + 1.0);
            }
            InfluenceType::Repulsive => {
                // Subtract from score for Repulsive objects, weighted by
                // repulsive_influence_multiplier
                score -= config.repulsive_influence_multiplier * ghost_influence.charge_value
                    / (distance2 + 1.0);
            }
        }
    }
    score
}

/// Calculates penalties for movement choices (walls, floor changes).
fn calculate_movement_penalties(
    potential_destination: Position,
    current_ghost_pos: &Position,
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
        penalty_score += FLOOR_CHANGE_PENALTY_BASE;
    }

    penalty_score
}

/// Spawns a `SaltyTrace` entity at the given `tile_position`.
pub(crate) fn spawn_salty_trace(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    tile_position: BoardPosition,
) {
    let mut pos = tile_position.to_position();
    let mut rng = random_seed::rng();
    pos.x += rng.random_range(-0.2..0.2);
    pos.y += rng.random_range(-0.2..0.2);
    pos.z += rng.random_range(-0.05..0.05); // Add small Z variation for traces
    commands
        .spawn(Sprite {
            image: asset_server.load("img/salt_particle.png"),
            color: css::DARK_GRAY.with_alpha(0.5).into(),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        })
        .insert(
            Transform::from_translation(perspective::to_screen_coord(pos))
                .with_scale(Vec3::new(0.5, 0.5, 0.5)),
        )
        .insert(pos)
        .insert(SaltyTrace)
        .insert(UVReactive(1.0))
        .insert(SaltyTraceTimer(Timer::from_seconds(600.0, TimerMode::Once)))
        .insert(MapColor {
            color: css::DARK_GRAY.with_alpha(0.5).into(),
        })
        .insert(GameSprite)
        .insert(SpriteLayer::default());
}
