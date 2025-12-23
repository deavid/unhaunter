use crate::components::ghost_behavior_dynamics::GhostBehaviorDynamics;
use crate::components::ghost_influence::{GhostInfluence, InfluenceType};
use bevy::color::palettes::css;
use bevy::prelude::*;
use rand::Rng;
use std::f64::consts::PI;
use uncore_board::components::mapcolor::MapColor;
use uncore_foundation::random_seed;
use uncore_foundation::utils::{MeanValue, PrintingTimer};
use uncore_resources::resources::board_data::BoardData;
use uncore_resources::resources::roomdb::RoomDB;
use uncore_resources::resources::summary_data::SummaryData;
use undifficulty::CurrentDifficulty;
use ungear::gear_stuff::GearStuff;
use ungearitems::components::sage::{SageSmokeParticle, SmokeParticleTimer};
use ungearitems::components::salt::{SaltyTrace, SaltyTraceTimer, UVReactive};
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;
use unmetrics::SendMetric;
use unplayer_core::resources::PlayerState;
use unrender::components::game::GameSprite;
use unrender::components::sprite_type::SpriteType;
use unspatial::{BoardPosition, Direction, Position};
use untags::PlayerTag;

use crate::metrics::{GHOST_ENRAGE, GHOST_MOVEMENT};
use uncore_events::events::ambient_sound_mute::AmbientSoundMuteEvent;

/// Enables/disables debug logs for hunting behavior.
const DEBUG_HUNTS: bool = true;

// Constants for movement penalties
const WALL_AVOIDANCE_PENALTY: f32 = -100.0; // Negative because it's added to score
const FLOOR_CHANGE_PENALTY_BASE: f32 = -50.0; // Negative, base penalty for changing floors

// New structures for refactored ghost behavior system

/// Decision about roar behavior with debugging information
#[derive(Debug)]
struct RoarDecision {
    roar_type: RoarType,
    should_play_now: bool,
    reason: RoarReason,
    time_override: Option<f32>,
}

/// Reason for roar decision (for debugging)
#[derive(Debug, Clone, Copy)]
enum RoarReason {
    HuntWarningStart,
    HuntingInProgress,
    RageBuildup,
    PeriodicSnore,
    None,
}

/// Result of rage calculation
#[derive(Debug)]
struct RageUpdateResult {
    rage_limit: f32,
    should_trigger_hunt: bool,
}

#[derive(Component)]
struct FadeOut {
    pub timer: Timer,
    pub roared: bool,
}

impl FadeOut {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            roared: false,
        }
    }
}

/// Updates the ghost's position based on its target location, hunting state, and
/// warping intensity.
///
/// This system handles the ghost's movement logic, ensuring it navigates the game
/// world according to its current state and objectives.
fn ghost_movement(
    mut q: Query<
        (&mut GhostSprite, &mut Position, Entity),
        (
            Without<PlayerTag>,
            Without<GhostInfluence>,
            Without<FadeOut>,
        ),
    >,
    qp: Query<&Position, With<PlayerTag>>,
    player_state: Res<PlayerState>,
    roomdb: Res<RoomDB>,
    mut summary: ResMut<SummaryData>,
    bf: Res<BoardData>,
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
                let player_pos_l: Vec<(&Position, bool)> = if player_state.health > 0.0 {
                    qp.iter()
                        .map(|pos| (pos, player_state.hiding_spot.is_some()))
                        .collect()
                } else {
                    Vec::new()
                };
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
                        + calculate_movement_penalties(candidate_dest, &pos, &bf, &difficulty)
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
                            && bf.collision_field[point_bpos.ndidx()].player_free
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
            if dstroom.is_some() && bf.collision_field[bpos.ndidx()].ghost_free {
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
                if bpos.is_valid(bf.map_size) && bf.collision_field[bpos.ndidx()].ghost_free {
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

#[derive(Debug, Clone, Copy)]
pub enum RoarType {
    Full,
    Dim,
    Snore,
    None,
}

impl RoarType {
    pub fn get_sound(&self) -> Option<String> {
        let roar_sounds = match self {
            RoarType::Full => vec![
                "sounds/ghost-roar-1.ogg",
                "sounds/ghost-roar-2.ogg",
                "sounds/ghost-roar-3.ogg",
                "sounds/ghost-roar-4.ogg",
            ],
            RoarType::Dim => vec![
                "sounds/ghost-effect-1.ogg",
                "sounds/ghost-effect-2.ogg",
                "sounds/ghost-effect-3.ogg",
                "sounds/ghost-effect-4.ogg",
            ],
            RoarType::Snore => vec![
                "sounds/ghost-snore-1.ogg",
                "sounds/ghost-snore-2.ogg",
                "sounds/ghost-snore-3.ogg",
                "sounds/ghost-snore-4.ogg",
            ],
            RoarType::None => return None,
        };

        roar_sounds
            .get(random_seed::rng().random_range(0..roar_sounds.len()))
            .map(|s| s.to_string())
    }

    pub fn get_volume(&self) -> f32 {
        match self {
            RoarType::Full => 1.0,
            RoarType::Dim => 0.9,
            RoarType::Snore => 0.3,
            RoarType::None => 0.0,
        }
    }
}

/// Manages the ghost's rage level, hunting behavior, and player interactions
/// during a hunt.
///
/// This system updates the ghost's rage based on player proximity, sanity, and
/// sound levels. It triggers hunts when rage exceeds a threshold and handles
/// player damage during hunts.
fn ghost_enrage(
    time: Res<Time>,
    mut timer: Local<PrintingTimer>,
    mut avg_angry: Local<MeanValue>,
    mut qg: Query<(&mut GhostSprite, &Position, &GhostBehaviorDynamics), Without<FadeOut>>,
    mut player_state: ResMut<PlayerState>,
    mut gs: GearStuff,
    mut last_roar: Local<f32>,
    difficulty: Res<CurrentDifficulty>,
    roomdb: Res<RoomDB>,
    mut ev_ambient_mute: MessageWriter<AmbientSoundMuteEvent>,
) {
    let measure = GHOST_ENRAGE.time_measure();

    timer.tick(time.delta());
    let dt = time.delta_secs();
    *last_roar += dt;

    for (mut ghost, ghost_position, dynamics) in qg.iter_mut() {
        // 1. Update basic timers
        update_ghost_timers_simple(&mut ghost, dt, &time);

        // 2. Handle salty trace spawning
        handle_salty_trace_spawning_simple(
            &mut ghost,
            ghost_position,
            &mut gs.commands,
            &gs.asset_server,
            &gs.bf,
        );

        // 3. Calculate minimum player distance for this ghost
        let min_player_dist = calculate_min_player_distance(ghost_position, &player_state);

        // 4. Apply distance-based effects
        apply_distance_based_effects(&mut ghost, min_player_dist, dt);

        // 5. Handle hunting phase
        if ghost.hunt_target {
            let hunt_result = handle_hunting_phase(
                &mut ghost,
                ghost_position,
                &mut player_state,
                &time,
                &difficulty,
                dt,
            );

            if hunt_result.should_roar {
                let roar_decision = RoarDecision {
                    roar_type: hunt_result.roar_type,
                    should_play_now: *last_roar > 3.0,
                    reason: RoarReason::HuntingInProgress,
                    time_override: None,
                };

                execute_roar_decision(&roar_decision, &mut last_roar, &mut gs, ghost_position);
            }
            continue;
        }

        // 6. Handle pre-warning and warning phases
        let warning_result = handle_warning_phases(&mut ghost, dt, &time, &mut ev_ambient_mute);

        // 7. Calculate rage
        let rage_result = calculate_rage_update(
            &mut ghost,
            ghost_position,
            &player_state,
            dynamics,
            &mut avg_angry,
            &difficulty,
            &roomdb,
            dt,
        );

        // 8. Check for hunt trigger
        if should_trigger_hunt(&ghost, &rage_result) {
            trigger_hunt_start(&mut ghost, &rage_result, &difficulty, &mut ev_ambient_mute);
        }

        // 9. Determine roar behavior
        let roar_decision =
            determine_roar_decision(&ghost, &rage_result, &warning_result, *last_roar, dt);

        // 9.5. Handle rage limit multiplier decay (for rage-based roars)
        if matches!(roar_decision.reason, RoarReason::RageBuildup)
            && ghost.rage_limit_multiplier > 1.0
        {
            ghost.rage_limit_multiplier /= 1.01_f32.powf(dt);
        }

        #[allow(clippy::explicit_auto_deref)]
        execute_roar_decision(&roar_decision, &mut *last_roar, &mut gs, ghost_position);

        // 10. Debug logging
        if timer.just_finished() && DEBUG_HUNTS {
            debug_log_ghost_state(&ghost, &rage_result, &roar_decision);
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
    bf: &Res<BoardData>,
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
    if !bf.collision_field[dest_bpos.ndidx()].player_free {
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
fn spawn_salty_trace(
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
            Transform::from_translation(pos.to_screen_coord()).with_scale(Vec3::new(0.5, 0.5, 0.5)),
        )
        .insert(pos)
        .insert(SaltyTrace)
        .insert(UVReactive(1.0))
        .insert(SaltyTraceTimer(Timer::from_seconds(600.0, TimerMode::Once)))
        .insert(MapColor {
            color: css::DARK_GRAY.with_alpha(0.5).into(),
        })
        .insert(GameSprite)
        .insert(SpriteType::Other);
}

fn ghost_fade_out_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(
        Entity,
        &mut FadeOut,
        &mut MapColor,
        &Position,
        Option<&GhostSprite>,
    )>,
    mut gs: GearStuff,
) {
    let mut rng = random_seed::rng();
    for (entity, mut fade_out, mut map_color, position, ghost_sprite) in query.iter_mut() {
        fade_out.timer.tick(time.delta());
        let rem_f = fade_out.timer.remaining_secs() / fade_out.timer.duration().as_secs_f32();

        // Fade out the sprite
        map_color.color.set_alpha(rem_f);

        // Emit smoke particles while fading
        if fade_out.timer.remaining_secs() > 0.0 && rng.random_bool(((1.0 - rem_f) / 3.0) as f64) {
            let pos = *position;
            commands
                .spawn(Sprite {
                    image: asset_server.load("img/smoke.png"),
                    color: Color::NONE,
                    ..default()
                })
                .insert(
                    Transform::from_translation(pos.to_screen_coord())
                        .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                )
                .insert(SageSmokeParticle)
                .insert(GameSprite)
                .insert(pos)
                .insert(Direction {
                    dx: rng.random_range(-0.9..0.9),
                    dy: rng.random_range(-0.9..0.9),
                    dz: rng.random_range(-0.5..0.5), // Add Z direction for smoke particles
                })
                .insert(MapColor {
                    color: Color::WHITE.with_alpha(0.20),
                })
                .insert(SmokeParticleTimer(Timer::from_seconds(
                    5.0,
                    TimerMode::Once,
                )))
                .insert(SpriteType::Other);
        }

        // Play roar sounds
        if let Some(_ghost_sprite) = ghost_sprite {
            if !fade_out.roared {
                // Play the first roar at 100% volume
                if let Some(roar_sound) = RoarType::Full.get_sound() {
                    gs.play_audio(roar_sound, 1.0, position);
                }
                fade_out.roared = true;
            } else if fade_out.timer.is_finished() {
                // Play the second roar at a lower volume
                if let Some(roar_sound) = RoarType::Full.get_sound() {
                    gs.play_audio(roar_sound, 0.2, position);
                }

                // Despawn the entity
                commands.entity(entity).despawn();
            }
        } else if fade_out.timer.is_finished() {
            // Despawn the breach when its timer is done
            commands.entity(entity).despawn();
        }
    }
}

/// Updates the ghost warning field based on the intensity of nearby ghosts.
///
/// This system calculates the ghost warning field based on the highest intensity
/// warning from any ghost. The warning field is used to display a visual warning
/// to the player when a ghost is nearby.
fn update_ghost_warning_field(
    mut haunt_state: ResMut<HauntState>,
    q_ghost: Query<(&GhostSprite, &Position)>,
    time: Res<Time>,
) {
    // Reset warning field
    haunt_state.ghost_warning_intensity = 0.0;
    haunt_state.ghost_warning_position = None;

    let mut max_intensity = 0.0;

    // Find the highest intensity warning from any ghost
    for (ghost, position) in q_ghost.iter() {
        if ghost.hunt_warning_intensity > max_intensity {
            max_intensity = ghost.hunt_warning_intensity;
            haunt_state.ghost_warning_position = Some(*position);
        }
    }

    let cur_t = time.elapsed_secs_f64();
    let wave = f64::sin(PI * cur_t * 2.0).powi(2);
    haunt_state.ghost_warning_intensity = max_intensity * wave as f32;
}

/// Calculate distance with Z component multiplied by 10 if on different floors
/// This makes the ghost less effective at damaging players across floors
fn calculate_weighted_distance(ghost_pos: &Position, player_pos: &Position) -> f32 {
    let dx = player_pos.x - ghost_pos.x;
    let dy = player_pos.y - ghost_pos.y;

    // Check if they're on different floors by comparing rounded Z values
    let ghost_floor = ghost_pos.z.round();
    let player_floor = player_pos.z.round();

    let dz = if ghost_floor != player_floor {
        // Multiply Z component by 10 when on different floors
        (player_pos.z - ghost_pos.z) * 10.0
    } else {
        player_pos.z - ghost_pos.z
    };

    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Calculate squared distance with Z component multiplied by 10 if on different floors
fn calculate_weighted_distance_squared(ghost_pos: &Position, player_pos: &Position) -> f32 {
    let dx = player_pos.x - ghost_pos.x;
    let dy = player_pos.y - ghost_pos.y;

    // Check if they're on different floors by comparing rounded Z values
    let ghost_floor = ghost_pos.z.round();
    let player_floor = player_pos.z.round();

    let dz = if ghost_floor != player_floor {
        // Multiply Z component by 10 when on different floors
        (player_pos.z - ghost_pos.z) * 10.0
    } else {
        player_pos.z - ghost_pos.z
    };

    dx * dx + dy * dy + dz * dz
}

/// Apply a visual "glitch" effect to ghosts by modifying their Transform scale
/// This creates a visual indication of ghost instability without affecting position or movement
fn ghost_scale_glitch_system(
    time: Res<Time>,
    mut q_ghost: Query<(&GhostSprite, &mut Transform), (With<GhostSprite>, Without<FadeOut>)>,
) {
    let mut rng = random_seed::rng();
    let dt = time.delta_secs();

    for (ghost, mut transform) in q_ghost.iter_mut() {
        if ghost.repellent_hits_delta > 0.0 {
            // Apply scale glitch based on repellent hits
            let glitch_intensity = ghost.repellent_hits_delta.clamp(0.0, 1.0);

            // Generate random scale variations
            let scale_x = 1.0 + rng.random_range(-glitch_intensity..glitch_intensity) * 0.8
                - glitch_intensity * 0.2;
            let scale_y = 1.0 + rng.random_range(-glitch_intensity..glitch_intensity) * 0.8;
            let scale_z = 1.0; // Keep Z scale consistent

            // Apply the glitch scale
            transform.scale = transform
                .scale
                .lerp(Vec3::new(scale_x, scale_y, scale_z), dt * 0.5);
        } else if ghost.repellent_misses_delta > 0.0 {
            let glitch_intensity = ghost.repellent_misses_delta.clamp(0.0, 1.0);
            // Generate random scale variations
            let scale_x = 1.0 + glitch_intensity * 0.15;
            let scale_y = 1.0 + glitch_intensity * 0.1;
            let scale_z = 1.0; // Keep Z scale consistent

            // Apply the glitch scale
            transform.scale = transform
                .scale
                .lerp(Vec3::new(scale_x, scale_y, scale_z), dt * 0.2);
        } else {
            // Restore normal scale when no glitch
            if transform.scale != Vec3::ONE {
                // Smoothly interpolate back to normal scale
                transform.scale = transform.scale.lerp(Vec3::ONE, dt * 0.5);

                // Snap to exactly 1.0 when very close to avoid floating point drift
                if (transform.scale - Vec3::ONE).length() < 0.01 {
                    transform.scale = Vec3::ONE;
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            ghost_movement,
            ghost_enrage,
            ghost_fade_out_system,
            update_ghost_warning_field,
            ghost_scale_glitch_system,
        ),
    );

    // Initialize dynamic behavior update system
    crate::systems::dynamic_behavior_update::app_setup(app);
}

// Helper functions for simplified ghost behavior processing

/// Updates basic ghost timers
fn update_ghost_timers_simple(ghost: &mut GhostSprite, dt: f32, time: &Res<Time>) {
    // Update calm time
    if ghost.calm_time_secs > 0.0 {
        ghost.calm_time_secs -= dt.min(ghost.calm_time_secs);
    }

    // Update salty effect timers if active
    if !ghost.salty_effect_timer.is_finished() && ghost.hunting <= 0.1 {
        ghost.salty_effect_timer.tick(time.delta());
        ghost.salty_trace_spawn_timer.tick(time.delta());
    }
}

/// Spawns salty traces when conditions are met
fn handle_salty_trace_spawning_simple(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    bf: &BoardData,
) {
    if !ghost.salty_effect_timer.is_finished()
        && ghost.hunting <= 0.1
        && ghost.salty_trace_spawn_timer.just_finished()
    {
        if random_seed::rng().random_bool(0.5) {
            // Find valid floor tile
            let ghost_board_position = ghost_position.to_board_position();
            let mut valid_tile = None;
            for nearby_tile in ghost_board_position.iter_xy_neighbors_nosize(1) {
                let collision_data = bf.collision_field[nearby_tile.ndidx()];
                if collision_data.player_free {
                    valid_tile = Some(nearby_tile);
                    break;
                }
            }

            if let Some(tile_position) = valid_tile {
                spawn_salty_trace(commands, asset_server, tile_position);
            }
        }
        ghost.salty_trace_spawn_timer.reset();
    }
}

/// Calculate minimum player distance to ghost
fn calculate_min_player_distance(ghost_position: &Position, player_state: &PlayerState) -> f32 {
    calculate_weighted_distance(ghost_position, &player_state.position).clamp(1.0, 1000.0)
}

/// Apply distance-based effects on ghost behavior
fn apply_distance_based_effects(ghost: &mut GhostSprite, min_player_dist: f32, dt: f32) {
    // Reduce ghost rage as player is further away
    ghost.rage -= dt * min_player_dist.sqrt() / 10.0;
    if !ghost.hunt_target {
        // Reduce ghost hunting when player is away
        ghost.hunting -= dt * min_player_dist.sqrt() / 3.0;
    }
}

/// Result of hunting phase processing
#[derive(Debug)]
struct HuntingResult {
    should_roar: bool,
    roar_type: RoarType,
}

/// Handle the hunting phase of ghost behavior
fn handle_hunting_phase(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    player_state: &mut PlayerState,
    time: &Res<Time>,
    difficulty: &Res<CurrentDifficulty>,
    dt: f32,
) -> HuntingResult {
    // Reset warning states during hunting
    ghost.hunt_warning_active = false;
    ghost.hunt_warning_intensity = 1.0;
    ghost.hunt_warning_timer = 0.0;

    let ghost_strength = (time.elapsed_secs() - ghost.hunt_time_secs).clamp(0.0, 2.0);

    // Apply player damage during hunt
    // For now, assume single player, damage based on distance to ghost
    let dist2 = calculate_weighted_distance_squared(ghost_position, &player_state.position) + 2.0;
    let dmg = dist2.recip() * difficulty.0.health_drain_rate;
    player_state.health -= dmg * dt * 30.0 * ghost_strength / (1.0 + ghost.calm_time_secs / 5.0);

    // Determine roar during hunting
    let roar_type = if ghost.hunting > 4.0 {
        RoarType::Full
    } else {
        RoarType::Dim
    };

    // Reduce rage during hunting
    ghost.rage -= dt * 20.0;
    if ghost.rage < 0.0 {
        ghost.rage = 0.0;
    }

    HuntingResult {
        should_roar: true,
        roar_type,
    }
}

/// Result of warning phase processing
#[derive(Debug)]
struct WarningResult {
    roar_triggered: bool,
    roar_type: RoarType,
    hunt_started: bool,
}

/// Handle pre-warning and warning phases
fn handle_warning_phases(
    ghost: &mut GhostSprite,
    dt: f32,
    time: &Res<Time>,
    ev_ambient_mute: &mut MessageWriter<AmbientSoundMuteEvent>,
) -> WarningResult {
    let mut result = WarningResult {
        roar_triggered: false,
        roar_type: RoarType::None,
        hunt_started: false,
    };

    // Pre-warning phase
    if ghost.pre_warning_timer > 0.0 {
        ghost.pre_warning_timer -= dt;
        if ghost.pre_warning_timer <= 0.0 {
            // Pre-warning timer expired, start actual hunt warning with roar
            if !ghost.hunt_warning_active {
                ghost.hunt_warning_active = true;
                ghost.hunt_warning_timer = 5.0;
                ghost.hunt_warning_intensity = 0.0;

                result.roar_triggered = true;
                result.roar_type = RoarType::Full;
            }
        }
    }

    // Warning phase
    if ghost.hunt_warning_active {
        ghost.hunt_warning_timer -= dt;
        ghost.hunt_warning_intensity = 1.0 - (ghost.hunt_warning_timer / 10.0);

        // Send stronger mute event when hunt is about to start (anticipatory)
        if ghost.hunt_warning_timer <= 0.5 && ghost.hunt_warning_timer > 0.5 - dt {
            // Send stronger/faster mute for actual hunt start
            ev_ambient_mute.write(AmbientSoundMuteEvent::default());

            // Trigger hunt after warning period
            ghost.hunt_warning_active = false;
            ghost.hunt_warning_intensity = 1.0; //Max intensity
            ghost.hunt_target = true;
            ghost.hunt_time_secs = time.elapsed_secs();
            warn!("Hunting player for {:.1}s", ghost.hunting);

            result.hunt_started = true;
        }
    } else if ghost.hunting < 0.001 {
        ghost.hunt_warning_intensity /= 2.2_f32.powf(dt);
    }

    result
}

/// Calculate rage update and return result information
fn calculate_rage_update(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    player_state: &PlayerState,
    dynamics: &GhostBehaviorDynamics,
    avg_angry: &mut MeanValue,
    difficulty: &Res<CurrentDifficulty>,
    roomdb: &Res<RoomDB>,
    dt: f32,
) -> RageUpdateResult {
    // Calculate player-induced rage
    let mut total_angry2 = 0.0;
    let mut player_in_room = false;
    let mut total_inv_sanity = 0.0;

    let sanity = player_state.sanity;
    let inv_sanity = (120.0 - sanity) / 100.0;

    let dist2 = calculate_weighted_distance_squared(ghost_position, &player_state.position)
        / difficulty.0.hunt_provocation_radius
        * (0.01 + sanity)
        + 0.1
        + sanity / 100.0;

    let angry2 = dist2.recip() * 1000000.0 / sanity
        * player_state.mean_sound
        * (player_state.health / 100.0).clamp(0.0, 1.0);

    total_angry2 += angry2 * inv_sanity + player_state.mean_sound.sqrt() * inv_sanity * dt * 3000.1;

    let player_board_position = player_state.position.to_board_position();
    if roomdb.room_tiles.contains_key(&player_board_position) {
        player_in_room = true;
        total_inv_sanity += inv_sanity;
    }

    let angry = total_angry2.sqrt();
    let a_f = 1.0 + (avg_angry.avg() * 2.0).powi(2);

    // Apply rage decay
    ghost.rage /= 1.01_f32.powf(dt / a_f);
    ghost.rage -= dt * 2.0 / a_f;
    if ghost.rage < 0.0 {
        ghost.rage = 0.0;
    }

    // Apply rage increases
    if player_in_room {
        ghost.rage += dt * difficulty.0.ghost_rage_likelihood * 5.2 * total_inv_sanity;
    }
    ghost.rage +=
        angry * dt / 10.0 / (1.0 + ghost.calm_time_secs) * difficulty.0.ghost_rage_likelihood;

    // Update hunting decay
    ghost.hunting -= dt * 0.2 / difficulty.0.ghost_hunt_duration;
    if ghost.hunting < 0.0 {
        ghost.hunting = 0.0;
    }

    avg_angry.push_len(angry, dt);

    // Calculate rage limit
    let rage_limit =
        400.0 * difficulty.0.ghost_rage_likelihood.sqrt() * ghost.rage_limit_multiplier
            / (dynamics.rage_tendency_multiplier + 1.01);
    ghost.rage_limit = rage_limit;

    // Determine if hunt should be triggered
    let should_trigger_hunt = ghost.rage > rage_limit
        && !ghost.hunt_warning_active
        && !ghost.hunt_target
        && ghost.pre_warning_timer <= 0.0;

    RageUpdateResult {
        rage_limit,
        should_trigger_hunt,
    }
}

/// Check if hunt should be triggered
fn should_trigger_hunt(_ghost: &GhostSprite, rage_result: &RageUpdateResult) -> bool {
    rage_result.should_trigger_hunt
}

/// Trigger the start of a hunt
fn trigger_hunt_start(
    ghost: &mut GhostSprite,
    _rage_result: &RageUpdateResult,
    difficulty: &Res<CurrentDifficulty>,
    ev_ambient_mute: &mut MessageWriter<AmbientSoundMuteEvent>,
) {
    // Start Pre-Warning Phase (anticipatory audio muting)
    ghost.pre_warning_timer = 3.0;
    ghost.rage_limit_multiplier *= 1.3;

    let prev_rage = ghost.rage;
    ghost.rage /= 1.0 + difficulty.0.ghost_hunt_cooldown;
    ghost.hunting += prev_rage / 50.0 + 5.0;
    ghost.hunt_warning_active = false;

    // Send anticipatory mute event BEFORE the hunt warning begins
    ev_ambient_mute.write(AmbientSoundMuteEvent::default());
}

/// Determine roar decision based on current state
fn determine_roar_decision(
    ghost: &GhostSprite,
    rage_result: &RageUpdateResult,
    warning_result: &WarningResult,
    last_roar_time: f32,
    _dt: f32,
) -> RoarDecision {
    // Handle warning-triggered roars first
    if warning_result.roar_triggered {
        return RoarDecision {
            roar_type: warning_result.roar_type,
            should_play_now: last_roar_time > 0.2,
            reason: RoarReason::HuntWarningStart,
            time_override: Some(0.2),
        };
    }

    // Check for rage-based roars when not hunting
    if ghost.rage > rage_result.rage_limit / 2.0 && ghost.hunting < 1.0 && last_roar_time > 10.0 {
        // Apply rage limit multiplier decay here instead of in main function
        if ghost.rage_limit_multiplier > 1.0 {
            // Note: we can't modify ghost here, so we'll handle this in the main function
        }
        return RoarDecision {
            roar_type: RoarType::Dim,
            should_play_now: true,
            reason: RoarReason::RageBuildup,
            time_override: None,
        };
    }

    // Periodic snore
    if last_roar_time > 30.0 {
        return RoarDecision {
            roar_type: RoarType::Snore,
            should_play_now: true,
            reason: RoarReason::PeriodicSnore,
            time_override: None,
        };
    }

    RoarDecision {
        roar_type: RoarType::None,
        should_play_now: false,
        reason: RoarReason::None,
        time_override: None,
    }
}

/// Execute a roar decision
fn execute_roar_decision(
    roar_decision: &RoarDecision,
    last_roar: &mut f32,
    gs: &mut GearStuff,
    ghost_position: &Position,
) {
    if roar_decision.should_play_now {
        let roar_time_threshold = roar_decision.time_override.unwrap_or(3.0);
        if *last_roar > roar_time_threshold
            && let Some(roar_sound) = roar_decision.roar_type.get_sound()
        {
            gs.play_audio(
                roar_sound,
                roar_decision.roar_type.get_volume(),
                ghost_position,
            );
            *last_roar = 0.0;

            if DEBUG_HUNTS {
                debug!(
                    "Ghost roar: {:?} - Reason: {:?}",
                    roar_decision.roar_type, roar_decision.reason
                );
            }
        }
    }
}

/// Debug logging for ghost state
fn debug_log_ghost_state(
    ghost: &GhostSprite,
    rage_result: &RageUpdateResult,
    roar_decision: &RoarDecision,
) {
    info!(
        "Ghost calm time: {:.1}, rage: {:.1}, rage limit: {:.1}, hunting: {:.1}, warn act: {}, warning int: {:.1}, warning timer: {:.1}, roar reason: {:?}",
        ghost.calm_time_secs,
        ghost.rage,
        rage_result.rage_limit,
        ghost.hunting,
        ghost.hunt_warning_active,
        ghost.hunt_warning_intensity,
        ghost.hunt_warning_timer,
        roar_decision.reason
    );
}
