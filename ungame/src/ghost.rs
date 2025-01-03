use uncore::components::game::GameSprite;
use uncore::components::ghost_influence::{GhostInfluence, InfluenceType};
use uncore::resources::boarddata::BoardData;
use uncore::resources::object_interaction::ObjectInteractionConfig;

use crate::board::{self, BoardPosition, Position};
use crate::difficulty::CurrentDifficulty;
use crate::gear::ext::systemparam::gearstuff::GearStuff;
use crate::gear::ext::types::items::sage::{SageSmokeParticle, SmokeParticleTimer};
use crate::ghost_definitions::GhostType;
use crate::maplight::MapColor;
use crate::player::{Hiding, PlayerSprite};
use crate::{summary, utils};
use bevy::color::palettes::css;
use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use std::time::Duration;

/// Enables/disables debug logs for hunting behavior.
const DEBUG_HUNTS: bool = false;

/// Represents a ghost entity in the game world.
///
/// This component stores the ghost's type, spawn point, target location,
/// interaction stats, current mood, hunting state, and other relevant attributes.
#[derive(Component, Debug)]
pub struct GhostSprite {
    /// The specific type of ghost, which determines its characteristics and abilities.
    pub class: GhostType,
    /// The ghost's designated spawn point (breach) on the game board.
    pub spawn_point: BoardPosition,
    /// The ghost's current target location in the game world. `None` if the ghost is
    /// wandering aimlessly.
    pub target_point: Option<Position>,
    /// Number of times the ghost has been hit with the correct type of repellent.
    pub repellent_hits: i64,
    /// Number of times the ghost has been hit with an incorrect type of repellent.
    pub repellent_misses: i64,
    /// Number of times the ghost has been hit with the correct type of repellent - in
    /// current frame.
    pub repellent_hits_frame: f32,
    /// Number of times the ghost has been hit with an incorrect type of repellent - in
    /// current frame.
    pub repellent_misses_frame: f32,
    /// The entity ID of the ghost's visual breach effect.
    pub breach_id: Option<Entity>,
    /// The ghost's current rage level, which influences its hunting behavior. Higher
    /// rage increases the likelihood of a hunt.
    pub rage: f32,
    /// The ghost's hunting state. A value greater than 0 indicates that the ghost is
    /// actively hunting a player.
    pub hunting: f32,
    /// Flag indicating whether the ghost is currently targeting a player during a hunt.
    pub hunt_target: bool,
    /// Time in seconds since the ghost started its current hunt.
    pub hunt_time_secs: f32,
    /// The ghost's current warping intensity, which affects its movement speed. Higher
    /// values result in faster warping.
    pub warp: f32,
    /// The ghost got hit by sage, and it will be calm for a while.
    pub calm_time_secs: f32,
    /// Timer to track the duration of the "Salty" side effect.
    pub salty_effect_timer: Timer,
    /// Timer to control the frequency of spawning Salty Traces.
    pub salty_trace_spawn_timer: Timer,
    /// Makes the ghost wait more for the next attack but it will be a harder attack.
    pub rage_limit_multiplier: f32,
}

#[derive(Component)]
pub struct FadeOut {
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

/// Marker component for the ghost's visual breach effect.
#[derive(Component, Debug)]
pub struct GhostBreach;

impl GhostSprite {
    /// Creates a new `GhostSprite` with a random `GhostType` and the specified spawn
    /// point.
    ///
    /// The ghost's initial mood, hunting state, and other attributes are set to
    /// default values.
    pub fn new(spawn_point: BoardPosition, ghost_types: &[GhostType]) -> Self {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..ghost_types.len());
        let class = ghost_types[idx];
        warn!("Ghost type: {:?} - {:?}", class, class.evidences());
        let mut salty_effect_timer = Timer::from_seconds(120.0, TimerMode::Once);
        salty_effect_timer.tick(Duration::from_secs(120));
        GhostSprite {
            class,
            spawn_point,
            target_point: None,
            repellent_hits: 0,
            repellent_misses: 0,
            repellent_hits_frame: 0.0,
            repellent_misses_frame: 0.0,
            breach_id: None,
            rage: 0.0,
            hunting: 0.0,
            hunt_target: false,
            hunt_time_secs: 0.0,
            warp: 0.0,
            calm_time_secs: 0.0,
            salty_effect_timer,
            salty_trace_spawn_timer: Timer::from_seconds(0.3, TimerMode::Repeating),
            rage_limit_multiplier: 1.0,
        }
    }

    /// Sets the `breach_id` field, associating the ghost with its visual breach effect.
    pub fn with_breachid(self, breach_id: Entity) -> Self {
        Self {
            breach_id: Some(breach_id),
            ..self
        }
    }
}

/// Updates the ghost's position based on its target location, hunting state, and
/// warping intensity.
///
/// This system handles the ghost's movement logic, ensuring it navigates the game
/// world according to its current state and objectives.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn ghost_movement(
    mut q: Query<
        (&mut GhostSprite, &mut Position, Entity),
        (
            Without<PlayerSprite>,
            Without<GhostInfluence>,
            Without<FadeOut>,
        ),
    >,
    qp: Query<(&Position, &PlayerSprite, Option<&Hiding>)>,
    roomdb: Res<crate::board::RoomDB>,
    mut summary: ResMut<summary::SummaryData>,
    bf: Res<BoardData>,
    mut commands: Commands,
    time: Res<Time>,
    config: Res<ObjectInteractionConfig>,
    object_query: Query<(&Position, &GhostInfluence)>,
    difficulty: Res<CurrentDifficulty>,
) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_secs() * 60.0;
    for (mut ghost, mut pos, entity) in q.iter_mut() {
        if let Some(target_point) = ghost.target_point {
            let mut delta = target_point.delta(*pos);
            if rng.gen_range(0..500) == 0 && delta.distance() > 3.0 && ghost.warp < 0.1 {
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
            }
            delta.dx *= ghost.warp + 1.0;
            delta.dy *= ghost.warp + 1.0;
            let mut finalize = false;
            if ghost.hunt_target {
                if time.elapsed_secs() - ghost.hunt_time_secs > 1.0 {
                    if dlen < 4.0 {
                        delta.dx /= (dlen + 1.5) / 4.0;
                        delta.dy /= (dlen + 1.5) / 4.0;
                    }
                    pos.x += delta.dx / 70.0 * dt * difficulty.0.ghost_hunting_aggression;
                    pos.y += delta.dy / 70.0 * dt * difficulty.0.ghost_hunting_aggression;
                    ghost.hunting -= dt / 60.0;
                }
                if ghost.hunting < 0.0 {
                    ghost.hunting = 0.0;
                    ghost.hunt_target = false;
                    finalize = true;
                    warn!("Hunt finished");
                }
            } else {
                pos.x += delta.dx / 200.0 * dt * difficulty.0.ghost_speed;
                pos.y += delta.dy / 200.0 * dt * difficulty.0.ghost_speed;
            }
            if dlen < 0.5 {
                finalize = true;
            }
            if finalize {
                ghost.target_point = None;
            }
        }
        if ghost.target_point.is_none() || (ghost.hunt_target && rng.gen_range(0..60) == 0) {
            let mut target_point = ghost.spawn_point.to_position();
            let wander: f32 = rng.gen_range(0.001..1.0_f32).powf(6.0) * 12.0 + 0.5;
            let dx: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
            let dy: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
            let dist: f32 = (0..5).map(|_| rng.gen_range(0.2..wander)).sum();
            let dd = (dx * dx + dy * dy).sqrt() / dist;
            let mut hunt = false;
            target_point.x = (target_point.x + pos.x * wander) / (1.0 + wander) + dx / dd;
            target_point.y = (target_point.y + pos.y * wander) / (1.0 + wander) + dy / dd;
            let ghbonus = if ghost.hunt_target { 10000.0 } else { 0.0001 };
            if rng.gen_range(0.0..(ghost.hunting * 10.0 + ghbonus).sqrt() * 10.0) > 10.0 {
                let player_pos_l: Vec<(&Position, Option<&Hiding>)> = qp
                    .iter()
                    .filter(|(_, p, _)| p.health > 0.0)
                    .map(|(pos, _, h)| (pos, h))
                    .collect();
                if !player_pos_l.is_empty() {
                    let idx = rng.gen_range(0..player_pos_l.len());
                    let (ppos, h) = player_pos_l[idx];
                    let search_radius = if h.is_some() { 2.0 } else { 1.0 };
                    let mut old_target = ghost.target_point.unwrap_or(*pos);
                    old_target.x += rng.gen_range(-search_radius..search_radius);
                    old_target.y += rng.gen_range(-search_radius..search_radius);
                    let ppos = if h.is_some() || ghost.calm_time_secs > 5.0 {
                        old_target
                    } else {
                        *ppos
                    };
                    ghost.calm_time_secs -= 2.0_f32.min(ghost.calm_time_secs);
                    let mut rng = rand::thread_rng();
                    let random_offset = Vec2::new(
                        rng.gen_range(-search_radius..search_radius),
                        rng.gen_range(-search_radius..search_radius),
                    );
                    target_point.x = ppos.x + random_offset.x;
                    target_point.y = ppos.y + random_offset.y;
                    hunt = true;
                }
            }

            // --- Sample Potential Destinations and Calculate Scores ---
            if !hunt {
                let mut potential_destinations: Vec<(f32, Position)> = Vec::new();
                for _ in 0..config.num_destination_points_to_sample {
                    let mut target_point = ghost.spawn_point.to_position();
                    let wander: f32 = rng.gen_range(0.001..1.0_f32).powf(6.0) * 12.0
                        / difficulty.0.ghost_attraction_to_breach
                        + 0.5;
                    let dx: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
                    let dy: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
                    let dist: f32 = (0..5).map(|_| rng.gen_range(0.2..wander)).sum();
                    let dd = (dx * dx + dy * dy).sqrt() / dist;
                    target_point.x = (target_point.x + pos.x * wander) / (1.0 + wander) + dx / dd;
                    target_point.y = (target_point.y + pos.y * wander) / (1.0 + wander) + dy / dd;
                    let score = 1.0
                        + calculate_destination_score(target_point, &object_query, &config)
                            / difficulty.0.ghost_attraction_to_breach;
                    potential_destinations.push((score, target_point));
                }

                // --- Select Destination with Highest Score --- Default to spawn point
                let mut best_destination = ghost.spawn_point.to_position();
                let mut best_score = f32::MIN;
                for (score, point) in potential_destinations {
                    if score > best_score {
                        best_score = score;
                        best_destination = point;
                    }
                }
                target_point = best_destination;
            }
            let bpos = target_point.to_board_position();
            let dstroom = roomdb.room_tiles.get(&bpos);
            if dstroom.is_some()
                && bf
                    .collision_field
                    .get(&bpos)
                    .map(|x| x.ghost_free)
                    .unwrap_or_default()
            {
                if hunt {
                    if !ghost.hunt_target {
                        ghost.hunt_time_secs = time.elapsed_secs();
                        warn!("Hunting player for {:.1}s", ghost.hunting);
                    }
                } else if ghost.hunt_target {
                    warn!("Hunt temporarily ended (remaining) {:.1}s", ghost.hunting);
                }
                ghost.target_point = Some(target_point);
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
        if ghost.repellent_hits > 1000 {
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
}

pub enum RoarType {
    Full,
    Dim,
    Snore,
    None,
}

impl RoarType {
    pub fn get_sound(&self) -> String {
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
            RoarType::None => vec![""],
        };
        let random_roar = roar_sounds[rand::thread_rng().gen_range(0..roar_sounds.len())];
        random_roar.to_string()
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
#[allow(clippy::too_many_arguments)]
fn ghost_enrage(
    time: Res<Time>,
    mut timer: Local<utils::PrintingTimer>,
    mut avg_angry: Local<utils::MeanValue>,
    mut qg: Query<(&mut GhostSprite, &Position), Without<FadeOut>>,
    mut qp: Query<(&mut PlayerSprite, &Position)>,
    mut gs: GearStuff,
    mut last_roar: Local<f32>,
    difficulty: Res<CurrentDifficulty>,
) {
    timer.tick(time.delta());
    let dt = time.delta_secs();
    for (mut ghost, ghost_position) in &mut qg {
        // --- Salty Trace Spawning Logic ---
        if !ghost.salty_effect_timer.finished() && ghost.hunting <= 0.1 {
            // Only spawn traces when NOT hunting and salty effect is active
            ghost.salty_effect_timer.tick(time.delta());
            ghost.salty_trace_spawn_timer.tick(time.delta());
            if ghost.salty_trace_spawn_timer.just_finished() {
                if rand::thread_rng().gen_bool(0.5) {
                    // 50% chance to spawn --- Find Valid Floor Tile ---
                    let ghost_board_position = ghost_position.to_board_position();
                    let mut valid_tile = None;
                    for nearby_tile in ghost_board_position.xy_neighbors(1) {
                        // Check adjacent tiles
                        if let Some(collision_data) = gs.bf.collision_field.get(&nearby_tile) {
                            if collision_data.player_free {
                                // Check if the tile is walkable
                                valid_tile = Some(nearby_tile);
                                break;
                            }
                        }
                    }

                    // --- Spawn SaltyTrace Entity ---
                    if let Some(tile_position) = valid_tile {
                        spawn_salty_trace(&mut gs.commands, &gs.asset_server, tile_position);
                    }
                }
                ghost.salty_trace_spawn_timer.reset();
            }
        }
    }
    *last_roar += dt;
    let mut should_roar = RoarType::None;
    let mut roar_time = 3.0;
    for (mut ghost, gpos) in &mut qg {
        if ghost.calm_time_secs > 0.0 {
            ghost.calm_time_secs -= dt.min(ghost.calm_time_secs);
        }

        // Calm ghost when players are far away
        let min_player_dist = qp
            .iter()
            .map(|(_, ppos)| OrderedFloat(gpos.distance(ppos)))
            .min()
            .unwrap_or(OrderedFloat(1000.0))
            .into_inner()
            .clamp(1.0, 1000.0);

        // Reduce ghost rage as player is further away
        ghost.rage -= dt * min_player_dist.sqrt() / 10.0;
        if !ghost.hunt_target {
            // Reduce ghost hunting when player is away
            ghost.hunting -= dt * min_player_dist.sqrt() / 3.0;
        }

        // ---
        if ghost.hunt_target {
            let ghost_strength = (time.elapsed_secs() - ghost.hunt_time_secs).clamp(0.0, 2.0);
            for (mut player, ppos) in &mut qp {
                let dist2 = gpos.distance2(ppos) + 2.0;
                let dmg = dist2.recip() * difficulty.0.health_drain_rate;
                player.health -=
                    dmg * dt * 30.0 * ghost_strength / (1.0 + ghost.calm_time_secs / 5.0);
            }
            if ghost.hunting > 4.0 {
                should_roar = RoarType::Full;
            } else {
                should_roar = RoarType::Dim;
            }
            ghost.rage -= dt * 20.0;
            if ghost.rage < 0.0 {
                ghost.rage = 0.0;
            }
            continue;
        }
        let mut total_angry2 = 0.0;
        for (player, ppos) in &qp {
            let sanity = player.sanity();
            let inv_sanity = (120.0 - sanity) / 100.0;
            let dist2 = gpos.distance2(ppos) / difficulty.0.hunt_provocation_radius
                * (0.01 + sanity)
                + 0.1
                + sanity / 100.0;
            let angry2 = dist2.recip() * 1000000.0 / sanity
                * player.mean_sound
                * (player.health / 100.0).clamp(0.0, 1.0);
            total_angry2 +=
                angry2 * inv_sanity + player.mean_sound.sqrt() * inv_sanity * dt * 3000.1;
        }
        let angry = total_angry2.sqrt();
        let a_f = 1.0 + (avg_angry.avg() * 2.0).powi(2);
        ghost.rage /= 1.01_f32.powf(dt / a_f);
        ghost.rage -= dt * 2.0 / a_f;
        if ghost.rage < 0.0 {
            ghost.rage = 0.0;
        }
        ghost.rage +=
            angry * dt / 10.0 / (1.0 + ghost.calm_time_secs) * difficulty.0.ghost_rage_likelihood;
        ghost.hunting -= dt * 0.2 / difficulty.0.ghost_hunt_duration;
        if ghost.hunting < 0.0 {
            ghost.hunting = 0.0;
        }
        avg_angry.push_len(angry, dt);
        if timer.just_finished() && DEBUG_HUNTS {
            dbg!(ghost.calm_time_secs, ghost.rage);
        }
        let rage_limit =
            400.0 * difficulty.0.ghost_rage_likelihood.sqrt() * ghost.rage_limit_multiplier;
        if ghost.rage > rage_limit {
            ghost.rage_limit_multiplier *= 1.3;
            let prev_rage = ghost.rage;
            ghost.rage /= 1.0 + difficulty.0.ghost_hunt_cooldown;
            if ghost.hunting < 1.0 {
                should_roar = RoarType::Full;
                roar_time = 0.2;
            }
            ghost.hunting += prev_rage / 50.0 + 5.0;
        } else if ghost.rage > rage_limit / 2.0 && ghost.hunting < 1.0 && roar_time > 10.0 {
            should_roar = RoarType::Dim;
            if ghost.rage_limit_multiplier > 1.0 {
                ghost.rage_limit_multiplier /= 1.01_f32.powf(dt);
            }
        }
        if *last_roar > 30.0 && matches!(should_roar, RoarType::None) {
            should_roar = RoarType::Snore;
        }
        if *last_roar > roar_time {
            let roar_sound = should_roar.get_sound();
            if !roar_sound.is_empty() {
                gs.play_audio(roar_sound, should_roar.get_volume(), gpos);
                *last_roar = 0.0;
            }
        }
    }
}

/// Calculates the desirability score of a potential destination point for the
/// ghost, considering the influence of nearby charged objects.
fn calculate_destination_score(
    potential_destination: Position,
    object_query: &Query<(&Position, &GhostInfluence)>,
    config: &Res<ObjectInteractionConfig>,
) -> f32 {
    let mut score = 0.0;

    // Iterate through objects with GhostInfluence
    for (object_position, ghost_influence) in object_query.iter() {
        let distance = potential_destination.distance(object_position);

        // Apply influence based on distance and charge value
        match ghost_influence.influence_type {
            InfluenceType::Attractive => {
                // Add to score for Attractive objects, weighted by attractive_influence_multiplier
                score += config.attractive_influence_multiplier * ghost_influence.charge_value
                    / (distance + 1.0);
            }
            InfluenceType::Repulsive => {
                // Subtract from score for Repulsive objects, weighted by
                // repulsive_influence_multiplier
                score -= config.repulsive_influence_multiplier * ghost_influence.charge_value
                    / (distance + 1.0);
            }
        }
    }
    score
}

/// Spawns a `SaltyTrace` entity at the given `tile_position`.
fn spawn_salty_trace(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    tile_position: BoardPosition,
) {
    use crate::gear::ext::types::items::salt::{SaltyTrace, SaltyTraceTimer, UVReactive};

    let mut pos = tile_position.to_position();
    let mut rng = rand::thread_rng();
    pos.x += rng.gen_range(-0.2..0.2);
    pos.y += rng.gen_range(-0.2..0.2);
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
        .insert(GameSprite);
}

#[allow(clippy::type_complexity)]
pub fn ghost_fade_out_system(
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
    let mut rng = rand::thread_rng();
    for (entity, mut fade_out, mut map_color, position, ghost_sprite) in query.iter_mut() {
        fade_out.timer.tick(time.delta());
        let rem_f = fade_out.timer.remaining_secs() / fade_out.timer.duration().as_secs_f32();

        // Fade out the sprite
        map_color.color.set_alpha(rem_f);

        // Emit smoke particles while fading
        if fade_out.timer.remaining_secs() > 0.0 && rng.gen_bool(((1.0 - rem_f) / 3.0) as f64) {
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
                .insert(board::Direction {
                    dx: rng.gen_range(-0.9..0.9),
                    dy: rng.gen_range(-0.9..0.9),
                    dz: 0.0,
                })
                .insert(MapColor {
                    color: Color::WHITE.with_alpha(0.20),
                })
                .insert(SmokeParticleTimer(Timer::from_seconds(
                    5.0,
                    TimerMode::Once,
                )));
        }

        // Play roar sounds
        if let Some(_ghost_sprite) = ghost_sprite {
            if !fade_out.roared {
                // Play the first roar at 100% volume
                gs.play_audio(RoarType::Full.get_sound(), 1.0, position);
                fade_out.roared = true;
            } else if fade_out.timer.finished() {
                // Play the second roar at a lower volume
                gs.play_audio(RoarType::Full.get_sound(), 0.2, position);

                // Despawn the entity
                commands.entity(entity).despawn();
            }
        } else if fade_out.timer.finished() {
            // Despawn the breach when its timer is done
            commands.entity(entity).despawn();
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (ghost_movement, ghost_enrage, ghost_fade_out_system),
    );
}
