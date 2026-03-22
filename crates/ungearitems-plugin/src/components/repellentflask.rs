use ndarray::Array3;
use unaudiospatial_core::emitter::AudioEmitter;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unfoundation_core::random_seed;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::types::gear::equipment::EquipmentPosition;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::components::repellent_particle::RepellentParticle;
use uninteraction_core::interaction::Triggered;
use unmetrics_core::metrics::SendMetric;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::Emissive;
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use unmission_core::summary::SummaryData;
use untypes_core::roles::LocalPlayerRole;
use untypes_core::states::AppState;

use crate::metrics;

use bevy::{color::palettes::css, prelude::*};
use rand::RngExt;
use ungear_core::types::gear::sprite_id::GearSpriteID;
pub(crate) use ungearitems_core::components::repellentflask::RepellentFlask;

// Colors for repellent particles
const ELECTRIC_BLUE: Color = Color::srgba(0.0, 0.3, 1.0, 1.0);
const BRIGHT_RED: Color = Color::srgba(1.0, 0.2, 0.0, 1.0);
use std::collections::HashMap;
use std::ops::{Add, Mul};

pub(crate) fn update_repellentflask_skeleton(
    mut q_repellent: Query<(Entity, &mut RepellentFlask), With<LocallyOwned>>,
    q_triggered: Query<&Triggered>,
    mut summary: ResMut<SummaryData>,
    mut commands: Commands,
    mut gs_audio: AudioEmitter,
) {
    for (entity, mut repellent) in q_repellent.iter_mut() {
        if q_triggered.get(entity).is_ok()
            && !repellent.active
            && repellent.qty > 0
            && repellent.liquid_content.is_some()
        {
            repellent.active = true;
            gs_audio.play_audio_nopos("sounds/spray.ogg".into(), 0.8);
            commands.entity(entity).remove::<Triggered>();
        }

        if repellent.active {
            let mut rng = random_seed::rng();
            if rng.random_range(0.0..1.0) <= 0.5 {
                if repellent.qty == RepellentFlask::MAX_QTY {
                    summary.repellent_used_amt += 1;
                }
                repellent.qty -= 1;
                if repellent.qty <= 0 {
                    repellent.qty = 0;
                    repellent.active = false;
                }
            }
        }
    }
}

pub(crate) fn update_repellentflask_skin(
    mut q_repellent: Query<(
        Entity,
        &RepellentFlask,
        &mut StatusText,
        &mut GearSprite,
        &Position,
        &EquipmentPosition,
    )>,
    mut commands: Commands,
    mut emitted_qty: Local<HashMap<Entity, i32>>,
) {
    for (entity, repellent, mut status, mut sprite, pos, ep) in q_repellent.iter_mut() {
        let prev_qty = emitted_qty.get(&entity).copied().unwrap_or(repellent.qty);
        if prev_qty > repellent.qty
            && repellent.qty >= 0
            && let Some(liquid_content) = repellent.liquid_content
        {
            let mut rng = random_seed::rng();
            for _ in repellent.qty..prev_qty {
                let mut particle_pos = *pos;
                particle_pos.z += 0.2;
                let spread: f32 = if matches!(ep, EquipmentPosition::Deployed) {
                    0.1
                } else {
                    0.4
                };
                particle_pos.x += rng.random_range(-spread..spread);
                particle_pos.y += rng.random_range(-spread..spread);

                commands
                    .spawn(Sprite {
                        color: Color::NONE,
                        ..default()
                    })
                    .insert(particle_pos)
                    .insert(GameSprite)
                    .insert(MapColor {
                        color: css::YELLOW.with_alpha(0.3).with_blue(0.02).into(),
                    })
                    .insert(Emissive {
                        color: css::YELLOW.into(),
                        intensity: 1.0,
                        light_reactivity: 2.0,
                        pulse_speed: 10.0,
                    })
                    .insert(RepellentParticle::new(liquid_content))
                    .insert(SpriteLayer::default());
            }
        }
        emitted_qty.insert(entity, repellent.qty);

        // Update StatusText
        let name = "Repellent";
        let status_line = if repellent.qty > 0 {
            match repellent.liquid_content {
                Some(gt) => format!("Anti-{}", gt.name()),
                None => "Empty (No Type)".to_string(),
            }
        } else {
            match repellent.liquid_content {
                Some(gt) => format!("Empty (was Anti-{})", gt.name()),
                None => "Empty".to_string(),
            }
        };

        let msg = if repellent.qty > 0 && repellent.liquid_content.is_some() {
            if repellent.active {
                let remaining = (repellent.qty as f32 * 0.5) as i32; // approximate
                format!("Emptying flask... {} units left", remaining)
            } else {
                "Flask ready.\nActivate near the Ghost.".to_string()
            }
        } else {
            "Flask empty.\nMust be filled on the van".to_string()
        };
        status.0 = format!("{name}: {status_line}\n{msg}");

        // Update GearSprite
        if repellent.liquid_content.is_some() && repellent.qty > 0 {
            sprite.0 = GearSpriteID::RepelentFlaskFull.to_visual_key();
        } else {
            sprite.0 = GearSpriteID::RepelentFlaskEmpty.to_visual_key();
        }
    }
}

fn repellent_update(
    mut cmd: Commands,
    mut qgs: Query<(&Position, &mut GhostSprite)>,
    mut qrp: Query<
        (
            &mut Position,
            &mut RepellentParticle,
            &mut MapColor,
            Entity,
            Option<&mut Emissive>,
        ),
        Without<GhostSprite>,
    >,
    bf: Res<BoardTopology>,
    bcf: Res<BoardCollisionField>,
    difficulty: Res<CurrentDifficulty>,
    mut positions: Local<Array3<Vec<Vec3>>>,
    mut positions_dirty: Local<Vec<(usize, usize, usize)>>,
    time: Res<Time>,
) {
    let measure = metrics::REPELLENT_UPDATE.time_measure();

    // Cleaning previous frame data relative to positions
    for idx in positions_dirty.drain(..) {
        if let Some(cell) = positions.get_mut(idx) {
            cell.clear();
        }
    }

    if qrp.is_empty() {
        return;
    }

    let mut rng = random_seed::rng();
    let dt = time.delta_secs();
    const SPREAD: f32 = 0.1;
    const SPREAD_SHORT: f32 = 0.02;
    if positions.dim() != bf.map_size {
        *positions = Array3::from_elem(bf.map_size, Vec::with_capacity(8));
    }

    // Collect particle positions per cell for same-cell repulsion
    for (r_pos, _rep, _, _, _) in &qrp {
        let bpos = r_pos.to_board_position();
        let nidx = bpos.ndidx();
        if let Some(cell) = positions.get_mut(nidx) {
            cell.push(r_pos.to_vec3());
            positions_dirty.push(nidx);
        }
    }

    for (mut r_pos, mut rep, mut mapcolor, entity, mut o_emissive) in &mut qrp {
        rep.life -= dt;
        if rep.life < 0.0 {
            cmd.entity(entity).despawn();
            continue;
        }
        let life_factor = rep.life_factor();
        let rev_factor = 1.01 - life_factor;
        let alpha = life_factor.cbrt() / 2.0 + 0.01;

        if rep.hit_correct {
            mapcolor.color = ELECTRIC_BLUE.with_alpha(alpha.cbrt());
            if let Some(ref mut emissive) = o_emissive {
                emissive.color = ELECTRIC_BLUE;
            }
        } else if rep.hit_incorrect {
            mapcolor.color = BRIGHT_RED.with_alpha(alpha.cbrt());
            if let Some(ref mut emissive) = o_emissive {
                emissive.color = BRIGHT_RED;
            }
        } else {
            mapcolor.color = RepellentParticle::DEFAULT_COLOR.with_alpha(alpha);
            if let Some(ref mut emissive) = o_emissive {
                emissive.color = css::YELLOW.into();
            }
        }

        if let Some(ref mut emissive) = o_emissive {
            emissive.intensity = alpha * 0.1;
        }

        let bpos = r_pos.to_board_position();
        let ndidx = bpos.ndidx();

        // Same-cell particle repulsion
        let mut total_force = Direction::zero();
        let v_pos = r_pos.to_vec3();
        for &s_p in positions[ndidx].iter() {
            let dist2 = v_pos.distance_squared(s_p) + 0.1;
            let delta = v_pos - s_p;
            let force = 4.0 * delta / dist2;
            total_force.dx += force.x;
            total_force.dy += force.y;
        }
        total_force.dx += rng.random_range(-0.1..0.1);
        total_force.dy += rng.random_range(-0.1..0.1);
        const PRESSURE_FORCE_SCALE: f32 = 1e-5;
        rep.dir = rep
            .dir
            .add(total_force.mul(PRESSURE_FORCE_SCALE))
            .mul(0.999);

        for nb in bpos.iter_xy_neighbors(1, bf.map_size) {
            let coll_tile_data = &bcf.0[nb.ndidx()];
            if !coll_tile_data.player_free && !coll_tile_data.see_through {
                // Collision with walls
                let wall_pos = nb.to_position();
                let delta = r_pos.delta(wall_pos);
                let dist2 = delta.distance2() + 0.2;
                let norm = delta.normalized();
                let recip = dist2.recip();
                let force = recip * 0.001;
                if bpos == nb {
                    rep.dir.dx *= 0.8;
                    rep.dir.dy *= 0.8;
                }
                rep.dir.dx += norm.dx * force;
                rep.dir.dy += norm.dy * force;
            }
        }
        r_pos.x += rng.random_range(-SPREAD..SPREAD) * rev_factor
            + rng.random_range(-SPREAD_SHORT..SPREAD_SHORT)
            + rep.dir.dx;
        r_pos.y += rng.random_range(-SPREAD..SPREAD) * rev_factor
            + rng.random_range(-SPREAD_SHORT..SPREAD_SHORT)
            + rep.dir.dy;
        r_pos.z += (rng.random_range(-SPREAD..SPREAD) * rev_factor
            + rng.random_range(-SPREAD_SHORT..SPREAD_SHORT))
            / 10.0;

        // Get the base floor height (integer part of z)
        let floor_height = r_pos.z.floor();
        // Apply the floating effect but preserve the floor level
        let particle_height = (r_pos.z - floor_height) * 100.0 + 0.5 * rep.life_factor();
        r_pos.z = floor_height + (particle_height / 101.0);

        if r_pos
            .to_board_position()
            .ndidx_checked_margin(bf.map_size)
            .is_none()
        {
            rep.life = 0.0;
        }
        for (g_pos, mut ghost) in &mut qgs {
            let dist2 = g_pos.distance2(&r_pos);
            if dist2 < 4.5 {
                let dist2b = (dist2 + 1.0) * 2.0;
                if ghost.class == rep.class {
                    ghost.repellent_hits_frame += dt * 180.2 / dist2b;
                    // Correct repellent - turn electric blue
                    rep.hit_correct = true;
                } else {
                    // Incorrect repellent - turn bright red
                    ghost.repellent_misses_frame += dt * 120.2 / dist2b;
                    rep.hit_incorrect = true;
                }
                rep.life -= 20.0 * dt / dist2b;
                // cmd.entity(entity).despawn();
            }
        }
    }
    for (_pos, mut ghost) in &mut qgs {
        if ghost.repellent_hits_frame >= 1.0 {
            while ghost.repellent_hits_frame >= 1.0 {
                ghost.repellent_hits += 1;
                ghost.repellent_hits_frame -= 1.0;
                ghost.rage += 0.6 * difficulty.0.ghost_rage_likelihood();
            }
            ghost.repellent_hits_delta = 1.0;
        } else {
            ghost.repellent_hits_frame = (ghost.repellent_hits_frame - dt).max(0.0);
            ghost.repellent_hits_delta -= dt;
            ghost.repellent_hits_delta = ghost
                .repellent_hits_delta
                .clamp(0.0, 1.0)
                .max(ghost.repellent_hits_frame);
        }
        if ghost.repellent_misses_frame >= 1.0 {
            while ghost.repellent_misses_frame >= 1.0 {
                ghost.repellent_misses += 1;
                ghost.repellent_misses_frame -= 1.0;
                ghost.rage += 0.6 * difficulty.0.ghost_rage_likelihood();
            }
            ghost.repellent_misses_delta = 1.0;
        } else {
            ghost.repellent_misses_frame = (ghost.repellent_misses_frame - dt).max(0.0);
            ghost.repellent_misses_delta -= dt;
            ghost.repellent_misses_delta = ghost
                .repellent_misses_delta
                .clamp(0.0, 1.0)
                .max(ghost.repellent_misses_frame);
        }
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_repellentflask_skeleton);
    app.add_systems(
        Update,
        (update_repellentflask_skin, repellent_update)
            .run_if(in_state(AppState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
}
