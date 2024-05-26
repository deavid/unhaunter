use crate::board::{BoardPosition, Position};
use crate::components::ghost_influence::GhostInfluence;
use crate::components::ghost_influence::InfluenceType;
use crate::ghost_definitions::GhostType;
use crate::object_interaction::ObjectInteractionConfig;
use crate::player::{Hiding, PlayerSprite};
use crate::{summary, utils};

use bevy::prelude::*;
use rand::Rng;

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
    /// The ghost's current target location in the game world. `None` if the ghost is wandering aimlessly.
    pub target_point: Option<Position>,
    /// Number of times the ghost has been hit with the correct type of repellent.
    pub repellent_hits: i64,
    /// Number of times the ghost has been hit with an incorrect type of repellent.
    pub repellent_misses: i64,
    /// The entity ID of the ghost's visual breach effect.
    pub breach_id: Option<Entity>,
    /// The ghost's current rage level, which influences its hunting behavior.
    /// Higher rage increases the likelihood of a hunt.
    pub rage: f32,
    /// The ghost's hunting state. A value greater than 0 indicates that the ghost is actively hunting a player.
    pub hunting: f32,
    /// Flag indicating whether the ghost is currently targeting a player during a hunt.
    pub hunt_target: bool,
    /// Time in seconds since the ghost started its current hunt.
    pub hunt_time_secs: f32,
    /// The ghost's current warping intensity, which affects its movement speed. Higher values result in faster warping.
    pub warp: f32,
}

/// Marker component for the ghost's visual breach effect.
#[derive(Component, Debug)]
pub struct GhostBreach;

impl GhostSprite {
    /// Creates a new `GhostSprite` with a random `GhostType` and the specified spawn point.
    ///
    /// The ghost's initial mood, hunting state, and other attributes are set to default values.
    pub fn new(spawn_point: BoardPosition) -> Self {
        let mut rng = rand::thread_rng();
        let ghost_types: Vec<_> = GhostType::all().collect();
        let idx = rng.gen_range(0..ghost_types.len());
        let class = ghost_types[idx];
        warn!("Ghost type: {:?} - {:?}", class, class.evidences());
        GhostSprite {
            class,
            spawn_point,
            target_point: None,
            repellent_hits: 0,
            repellent_misses: 0,
            breach_id: None,
            rage: 0.0,
            hunting: 0.0,
            hunt_target: false,
            hunt_time_secs: 0.0,
            warp: 0.0,
        }
    }

    ///  Sets the `breach_id` field, associating the ghost with its visual breach effect.
    pub fn with_breachid(self, breach_id: Entity) -> Self {
        Self {
            breach_id: Some(breach_id),
            ..self
        }
    }
}

/// Updates the ghost's position based on its target location, hunting state, and warping intensity.
///
/// This system handles the ghost's movement logic, ensuring it navigates the game world according to its
/// current state and objectives.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn ghost_movement(
    mut q: Query<
        (&mut GhostSprite, &mut Position, Entity),
        (Without<PlayerSprite>, Without<GhostInfluence>),
    >,
    qp: Query<(&Position, &PlayerSprite, Option<&Hiding>)>,
    roomdb: Res<crate::board::RoomDB>,
    mut summary: ResMut<summary::SummaryData>,
    bf: Res<crate::board::BoardData>,
    mut commands: Commands,
    time: Res<Time>,
    config: Res<ObjectInteractionConfig>,
    object_query: Query<(&Position, &GhostInfluence)>,
) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_seconds() * 60.0;
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
                if time.elapsed_seconds() - ghost.hunt_time_secs > 1.0 {
                    if dlen < 4.0 {
                        delta.dx /= (dlen + 1.5) / 4.0;
                        delta.dy /= (dlen + 1.5) / 4.0;
                    }
                    pos.x += delta.dx / 70.0 * dt;
                    pos.y += delta.dy / 70.0 * dt;
                    ghost.hunting -= dt / 60.0;
                }
                if ghost.hunting < 0.0 {
                    ghost.hunting = 0.0;
                    ghost.hunt_target = false;
                    finalize = true;
                    warn!("Hunt finished");
                }
            } else {
                pos.x += delta.dx / 200.0 * dt;
                pos.y += delta.dy / 200.0 * dt;
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
            let wander: f32 = rng.gen_range(0.0..1.0_f32).powf(6.0) * 12.0 + 0.5;
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
                    let ppos = if h.is_some() { old_target } else { *ppos };

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
                    let wander: f32 = rng.gen_range(0.0..1.0_f32).powf(6.0) * 12.0 + 0.5;
                    let dx: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
                    let dy: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
                    let dist: f32 = (0..5).map(|_| rng.gen_range(0.2..wander)).sum();
                    let dd = (dx * dx + dy * dy).sqrt() / dist;
                    target_point.x = (target_point.x + pos.x * wander) / (1.0 + wander) + dx / dd;
                    target_point.y = (target_point.y + pos.y * wander) / (1.0 + wander) + dy / dd;

                    let score = calculate_destination_score(target_point, &object_query, &config);
                    potential_destinations.push((score, target_point));
                }

                // --- Select Destination with Highest Score ---
                let mut best_destination = ghost.spawn_point.to_position(); // Default to spawn point
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
                        ghost.hunt_time_secs = time.elapsed_seconds();
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
        if ghost.repellent_hits > 100 {
            summary.ghosts_unhaunted += 1;
            if let Some(breach) = ghost.breach_id {
                commands.entity(breach).despawn_recursive();
            }
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Manages the ghost's rage level, hunting behavior, and player interactions during a hunt.
///
/// This system updates the ghost's rage based on player proximity, sanity, and sound levels.
/// It triggers hunts when rage exceeds a threshold and handles player damage during hunts.
fn ghost_enrage(
    time: Res<Time>,
    mut timer: Local<utils::PrintingTimer>,
    mut avg_angry: Local<utils::MeanValue>,
    mut qg: Query<(&mut GhostSprite, &Position)>,
    mut qp: Query<(&mut PlayerSprite, &Position)>,
) {
    timer.tick(time.delta());
    let dt = time.delta_seconds();

    for (mut ghost, gpos) in &mut qg {
        if ghost.hunt_target {
            let ghost_strength = (time.elapsed_seconds() - ghost.hunt_time_secs).clamp(0.0, 2.0);
            for (mut player, ppos) in &mut qp {
                let dist2 = gpos.distance2(ppos) + 2.0;
                let dmg = dist2.recip();
                player.health -= dmg * dt * 30.0 * ghost_strength;
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
            let dist2 = gpos.distance2(ppos) * (0.01 + sanity) + 0.1 + sanity / 100.0;
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
        ghost.rage += angry * dt / 10.0;
        ghost.hunting -= dt * 0.2;
        if ghost.hunting < 0.0 {
            ghost.hunting = 0.0;
        }

        avg_angry.push_len(angry, dt);
        if timer.just_finished() && DEBUG_HUNTS {
            dbg!(&avg_angry.avg(), ghost.rage);
        }
        let rage_limit = if DEBUG_HUNTS { 60.0 } else { 400.0 };
        if ghost.rage > rage_limit {
            let prev_rage = ghost.rage;
            ghost.rage /= 3.0;
            ghost.hunting += (prev_rage - ghost.rage) / 10.0 + 5.0;
        }
    }
}

/// Calculates the desirability score of a potential destination point for the ghost,
/// considering the influence of nearby charged objects.
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
                // Subtract from score for Repulsive objects, weighted by repulsive_influence_multiplier
                score -= config.repulsive_influence_multiplier * ghost_influence.charge_value
                    / (distance + 1.0);
            }
        }
    }

    score
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, (ghost_movement, ghost_enrage));
}
