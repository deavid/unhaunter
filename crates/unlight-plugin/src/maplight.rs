//! ## Map Lighting and Visibility Module
//!
//! This module handles lighting, visibility, and color calculations for the game
//! world. It includes:
//!
//! * Functions for calculating the player's visibility field based on line-of-sight
//!   and potentially sanity levels.
//!
//! * Functions for applying lighting effects to map tiles and sprites, simulating
//!   various light sources (ambient, flashlight, ghost effects) and adjusting colors
//!   based on visibility and exposure.
//!
//! * Systems for dynamically updating lighting and visibility as the player moves and
//!   interacts with the environment.
use bevy::ecs::system::SystemParam;
use bevy::{color::palettes::css, prelude::*};
use bevy_platform::collections::HashMap;
use bevy_platform::collections::HashSet;
use core::f32;
use ndarray::Array3;
use rand::Rng;
use std::collections::VecDeque;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::roomdb::RoomDB;
pub(crate) use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unboard_core::types::fielddata::CollisionFieldData;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unlight_core::resources::light_grid::LightGrid;
pub(crate) use unlight_core::types::light::LightData;
use unrender_std::utils::perspective;
use unsound_core::resources::SoundGrid;
use unspatial_core::orientation::Orientation;
use unthermal_core::resources::ThermalGrid;
use unui_core::resources::MouseVisibility;

#[derive(SystemParam)]
pub(crate) struct GridResources<'w> {
    bf: Res<'w, BoardTopology>,
    bef: Res<'w, BoardEntityField>,
    bcf: Res<'w, BoardCollisionField>,
    tg: Res<'w, ThermalGrid>,
    sg: Res<'w, SoundGrid>,
    miasma: Res<'w, MiasmaGrid>,
    miasma_config: Res<'w, MiasmaConfig>,
}
use unfog_core::components::MiasmaSprite;
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;
use unfoundation_core::platform::plt::IS_WASM;
pub(crate) use unfoundation_core::types::light::LightType;
use unfoundation_core::utils::temperature::kelvin_to_celsius;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::EquipmentPosition;
use ungear_core::types::gear::Hand;
use ungearitems_core::components::salt::UVReactive;
// use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::MainPlayer;
use unrender_std::components::game::MapTileSprite;
use unrender_std::components::light::LightEmitter;
use unrender_std::components::visuals::SpectralClarity;
use unrender_std::components::visuals::{
    AlphaModulator, EctoplasmVisuals, Ethereal, InfraredSensitive, LightSensitive, Luminescent,
    ShadowCaster, SpectralInfluence, SpectralInfluenceType, UltravioletSensitive, Viewer,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::light::{compute_color_exposure, lerp_color};
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use untypes_core::difficulty::Difficulty;

use crate::metrics::{APPLY_LIGHTING, COMPUTE_VISIBILITY, PLAYER_VISIBILITY};
use unfoundation_core::random_seed;

/// Computes the player's visibility field, determining which areas of the map are
/// visible.
///
/// This function uses a line-of-sight algorithm to calculate visibility, taking
/// into account walls, obstacles, and potentially the player's sanity level. The
/// visibility field is stored in a `HashMap`, where the keys are `BoardPosition`s
/// and the values are visibility factors (0.0 to 1.0).
pub(crate) fn compute_visibility(
    vis_field: &mut Array3<f32>,
    collision_field: &Array3<CollisionFieldData>,
    pos_start: &Position,
    roomdb: Option<&mut RoomDB>,
    pre_fill: bool,
) {
    let measure = COMPUTE_VISIBILITY.time_measure();
    if pre_fill {
        vis_field.fill(-0.001);
    }
    let mut queue = VecDeque::with_capacity(256);
    let start = pos_start.to_board_position();
    let map_size = collision_field.dim();
    const Z_FACTOR: f32 = 2.0;
    queue.push_front((start.clone(), start.clone()));
    vis_field[start.ndidx()] = 1.0;
    while let Some((pos, pos2)) = queue.pop_back() {
        let pds = pos.to_position().distance_zf(pos_start, Z_FACTOR);
        let p = pos.ndidx();
        let src_f = vis_field[p];
        let cf = &collision_field[p];
        if !(cf.player_free || cf.see_through) {
            // If the current position analyzed is not free (a wall or out of bounds) then
            // stop extending.
            continue;
        }

        let neighbors = pos.iter_xy_neighbors(1, map_size);
        for npos in neighbors {
            if npos == pos {
                continue;
            }
            let np = npos.ndidx();
            let ncf = collision_field[np];
            let npds = npos.to_position().distance_zf(pos_start, Z_FACTOR);
            let npref = npos.distance(&pos2) / 2.0;
            let f = if npds < 1.5 {
                1.0
            } else {
                ((npds - pds) / npref).clamp(0.0, 1.0).powf(2.0)
            };
            let mut dst_f = src_f * f;
            if dst_f < 0.00001 {
                continue;
            }
            let k = if let Some(roomdb) = roomdb.as_ref() {
                match roomdb.room_tiles.get(&npos).is_some() {
                    // Decrease view range inside the location
                    true => 6.0,
                    false => 8.0,
                }
            } else {
                // For deployed gear
                3.0
            };
            dst_f /= 1.0 + ((npds - 1.5) / k).clamp(0.0, 6.0);
            let vf_np = &mut vis_field[np];
            // Apply a visibility penalty to collision tiles that are in positive X or Y direction
            let mut visibility_factor = 1.0;
            if !(ncf.player_free || ncf.see_through) {
                // Check if the neighbor is in positive X or Y direction relative to current tile
                if npos.x > pos.x || npos.y < pos.y {
                    if ncf.is_dynamic {
                        visibility_factor = (0.5 / (npds + 1.0)).cbrt(); // Doors have less penalty
                    } else {
                        visibility_factor = 0.2 / (npds + 1.0); // Apply the penalization
                    }
                }
            }

            if *vf_np < -0.000001 {
                if ncf.player_free || ncf.see_through {
                    queue.push_front((npos.clone(), pos.clone()));
                }
                *vf_np = dst_f * visibility_factor;
            } else {
                *vf_np = 1.0 - (1.0 - *vf_np) * (1.0 - dst_f * visibility_factor);
            }
            if ncf.stair_offset != 0 && start.z == npos.z {
                // Move up/down stairs too
                let n2pos = BoardPosition {
                    x: npos.x,
                    y: npos.y,
                    z: npos.z + ncf.stair_offset as i64,
                };
                let pos2 = BoardPosition {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z + ncf.stair_offset as i64,
                };
                let vf_np = &mut vis_field[n2pos.ndidx()];
                if *vf_np < -0.000001 {
                    *vf_np = dst_f / 10.0;
                    // info!("stair: {:?}", n2pos);
                    queue.push_front((n2pos, pos2));
                }
            }
        }
    }
    measure.end_ms();
}

/// System to calculate the player's visibility field and update VisibilityData.
pub(crate) fn player_visibility_system(
    mut q_vf: Query<(&Position, &mut VisibilityData), With<Viewer>>,
    bcf: Res<BoardCollisionField>,
    mut roomdb: ResMut<RoomDB>,
) {
    let measure = PLAYER_VISIBILITY.time_measure();

    for (pos, mut vf) in q_vf.iter_mut() {
        if vf.visibility_field.dim() != bcf.0.dim() {
            vf.visibility_field = Array3::from_elem(bcf.0.dim(), -0.001_f32);
        } else {
            vf.visibility_field.fill(-0.001_f32);
        }
        // Calculate visibility
        compute_visibility(
            &mut vf.visibility_field,
            &bcf.0,
            pos,
            Some(&mut roomdb),
            false,
        );
    }
    measure.end_ms();
}

/// Applies lighting effects to map tiles and sprites, adjusting colors based on
/// visibility and exposure.
///
/// This function:
///
/// * Simulates lighting from various sources (ambient light, flashlights, ghost
///   effects).
///
/// * Calculates the relative exposure based on light levels and the player's current
///   exposure adaptation.
///
/// * Adjusts tile and sprite colors based on lighting, visibility, and exposure,
///   creating a realistic and atmospheric visual experience.
#[expect(clippy::type_complexity)]
pub(crate) fn apply_lighting(
    mut qt2: Query<
        (
            Entity,
            &Position,
            &MeshMaterial2d<CustomMaterial1>,
            &mut Visibility,
            Option<&Behavior>,
            Option<&SpectralInfluence>,
            Option<&Interactive>,
            Option<&Ethereal>,
            Option<&EctoplasmVisuals>,
            Option<&SpectralClarity>,
            (
                Option<&LightSensitive>,
                Option<&InfraredSensitive>,
                Option<&UltravioletSensitive>,
                Option<&MapColor>,
                Option<&MiasmaSprite>,
                Option<&AlphaModulator>,
            ),
        ),
        With<MapTileSprite>,
    >,
    materials1: ResMut<Assets<CustomMaterial1>>,
    qp: Query<(&Position, &Viewer, &Direction, &PlayerGear, Has<MainPlayer>)>,
    q_deployed: Query<(&Position, &DeployedGear, &LightEmitter, &Toggleable)>,
    q_flashlight: Query<(&LightEmitter, &Toggleable)>,
    mut lg: ResMut<LightGrid>,
    grids: GridResources,
    // haunt_state: Res<HauntState>,
    q_vf: Query<&VisibilityData, With<MainPlayer>>,
    time: Res<Time>,
    mut sprite_set: ParamSet<(
        Query<
            (
                Entity,
                &Position,
                Option<&mut Sprite>,
                Option<&MeshMaterial2d<CustomMaterial1>>,
                Option<&LightSensitive>,
                Option<&UltravioletSensitive>,
                Option<&InfraredSensitive>,
                Option<&EctoplasmVisuals>,
                Option<&Ethereal>,
                Option<&Luminescent>,
                Option<&ShadowCaster>,
                Option<&MapColor>,
                Option<&UVReactive>,
                Option<&MiasmaSprite>,
                Option<&SpectralClarity>,
            ),
            Without<MapTileSprite>,
        >,
        Query<
            (&Position, &mut Sprite),
            (
                With<MapTileSprite>,
                Without<Ethereal>,
                Without<ShadowCaster>,
            ),
        >,
    )>,
    difficulty: Res<CurrentDifficulty>,
    mut visible: Local<HashSet<Entity>>,
    mouse_visibility: Res<MouseVisibility>,
) {
    let bf = &grids.bf;
    let bef = &grids.bef;
    let bcf = &grids.bcf;
    let tg = &grids.tg;
    let sg = &grids.sg;
    let miasma = &grids.miasma;
    let miasma_config = &grids.miasma_config;

    let measure = APPLY_LIGHTING.time_measure();

    let Ok(vf) = q_vf.single() else {
        return;
    };

    let mut rng = random_seed::rng();

    // Difficulty-based ambient light boost for tutorials.
    // 1.0 for Tutorial 1, 0.0 for Standard+.
    let tutorial_light_factor = match difficulty.0.difficulty {
        Difficulty::TutorialChapter1 => 1.0,
        Difficulty::TutorialChapter2 => 0.8,
        Difficulty::TutorialChapter3 => 0.6,
        Difficulty::TutorialChapter4 => 0.4,
        Difficulty::TutorialChapter5 => 0.2,
        _ => 0.0,
    };

    let mut cursor_exp: f32 = 0.0;
    let mut exp_count: f32 = 0.0001;
    let mut flashlights: Vec<(&Position, Direction, f32, Color, LightType, Array3<f32>)> = vec![];
    let mut player_pos = Position::new_i64(0, 0, 0);
    let elapsed = time.elapsed_secs();

    let board_dim = bcf.0.dim();
    if bf.map_size.0 == 0 {
        // If we don't have a valid map, skip this
        return;
    }

    // Weight highlights more when calculating exposure (Power Average)
    const HIGHLIGHT_PRIORITY_POWER: f32 = 1.0;

    // Check if visibility field is properly initialized
    if vf.visibility_field.is_empty() {
        return;
    }
    const FLASHLIGHT_POWER_FACTOR: f32 = 0.1;
    // Deployed gear
    for (pos, deployed_gear, fl, toggle) in q_deployed.iter() {
        if !toggle.is_on {
            continue;
        }
        let power = fl.power;
        let color = fl.color;
        let light_type = fl.light_type;

        if power > 0.0 {
            let vis_field: Array3<f32> = Array3::from_elem(board_dim, -0.001_f32);
            flashlights.push((
                pos,
                deployed_gear.direction,
                power * FLASHLIGHT_POWER_FACTOR,
                color,
                light_type,
                vis_field,
            ));
        }
    }
    for (pos, _viewer, direction, gear, is_main_player) in qp.iter() {
        let mut player_flashlight: Vec<(f32, Color, EquipmentPosition, LightType)> = vec![];

        let mut check_gear = |entity: Entity, p: EquipmentPosition| {
            if let Ok((fl, toggle)) = q_flashlight.get(entity)
                && toggle.is_on
            {
                player_flashlight.push((
                    fl.power * FLASHLIGHT_POWER_FACTOR,
                    fl.color,
                    p,
                    fl.light_type,
                ));
            }
        };

        if let Some(e) = gear.left_hand {
            check_gear(e, EquipmentPosition::Hand(Hand::Left));
        }
        if let Some(e) = gear.right_hand {
            check_gear(e, EquipmentPosition::Hand(Hand::Right));
        }
        for e in &gear.inventory {
            check_gear(*e, EquipmentPosition::Stowed);
        }

        for (power, color, p, light_type) in player_flashlight {
            if power > 0.0 {
                use EquipmentPosition::*;

                let mut fldir = *direction;
                if p == Stowed {
                    fldir = Direction {
                        dx: fldir.dx / 1000.0,
                        dy: fldir.dy / 1000.0,
                        dz: fldir.dz / 1000.0,
                    };
                }
                let vis_field: Array3<f32> = Array3::from_elem(board_dim, -0.001_f32);
                flashlights.push((pos, fldir, power, color, light_type, vis_field));
            }
        }
        if !is_main_player {
            continue;
        }

        let cursor_pos = pos.to_board_position();
        for npos in cursor_pos.iter_xy_neighbors(10, board_dim) {
            let lf = &lg.light_field[npos.ndidx()];
            let vis = vf.visibility_field[npos.ndidx()].max(0.00001);

            // Power average to prioritize highlights in the field of view.
            cursor_exp += lf.lux.powf(HIGHLIGHT_PRIORITY_POWER) * vis;
            exp_count += vis;
        }
        player_pos = *pos;
    }
    for (pos, _fldir, _power, _color, _light_type, vis_field) in flashlights.iter_mut() {
        compute_visibility(vis_field, &bcf.0, pos, None, false);
    }

    // --- Access queries from the ParamSet ---
    let mut tile_sprites = sprite_set.p1();

    // --- Highlight placement tiles ---
    for (player_pos, _, _, player_gear, is_main_player) in qp.iter() {
        if is_main_player && player_gear.held_item.is_some() {
            // Only highlight if the player is holding an object
            let target_tile = player_pos.to_board_position();
            for (tile_pos, mut sprite) in tile_sprites.iter_mut() {
                // Removed 'behavior' from the loop
                if tile_pos.to_board_position() == target_tile {
                    // Removed walkable check Adjust highlight color and intensity as needed
                    let highlight_color = Color::srgba(0.0, 1.0, 0.0, 0.3);
                    sprite.color = lerp_color(sprite.color, highlight_color, 0.5);
                }
            }
        }
    }
    let mut qt = sprite_set.p0();
    cursor_exp = (cursor_exp / exp_count).powf(HIGHLIGHT_PRIORITY_POWER.recip());
    // Account for the eye seeing the flashlight on.
    // TODO: Account this from the player's perspective as the payer torch might
    // be off but someother player might have it on.
    let fl_total_power: f32 = flashlights
        .iter()
        .map(|x| {
            let mut power = x.2;
            power *= match x.4 {
                LightType::Visible => 1.0,
                LightType::Red => 0.003,
                LightType::InfraRedNV => 0.5,
                LightType::UltraViolet => 0.5,
            };
            power / (player_pos.distance2(x.0) + 1.0)
        })
        .sum();
    cursor_exp += fl_total_power.sqrt();

    // FIR Filter with Hann Window (240 frames)
    lg.exposure_history.push_back(cursor_exp);
    while lg.exposure_history.len() > 240 {
        lg.exposure_history.pop_front();
    }
    let mut sum_weights = 0.0;
    let mut sum_values = 0.0;
    for (i, &v) in lg.exposure_history.iter().enumerate() {
        let weight = lg.exposure_weights.get(i).copied().unwrap_or(1.0);
        sum_values += v * weight;
        sum_weights += weight;
    }
    cursor_exp = sum_values / (sum_weights + 0.001);
    lg.exposure_lux = cursor_exp;

    cursor_exp = cursor_exp.clamp(0.0, 100.0);
    // Darken picture without touching night vision
    cursor_exp += cursor_exp.sqrt() * 2.0;
    // Minimum exp - controls how dark we can see
    cursor_exp = cursor_exp.clamp(0.40, 100.0);

    if !cursor_exp.is_normal() {
        warn!("cursor_exp is not 'normal': {}", cursor_exp);
        cursor_exp = lg.current_exposure;
    }

    // Additional IIR filter: 63.2% in 2 seconds (tau = 2s)
    let dt = time.delta_secs();
    let alpha = 1.0 - (-dt / 2.0).exp();
    lg.current_exposure = lg.current_exposure * (1.0 - alpha) + cursor_exp * alpha;

    let exposure = lg.current_exposure;
    let raw_lux = lg.exposure_history.back().copied().unwrap_or(0.0);

    let mut lightdata_map: HashMap<BoardPosition, LightData> = HashMap::new();

    // Primes: 13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,173,281,409,541,659,809
    const VSMALL_PRIME: usize = 59;
    const BIG_PRIME: usize = 95629;
    let mask: usize = rng.random_range(0..usize::MAX);
    let lf = &lg.light_field;

    // --- Shared Lighting Sampling Logic ---

    // minimum distance for flashlight
    const FL_MIN_DST: f32 = 0.1;

    let fpos_gamma_color = |target_pos: Position,
                            is_light_sensitive: bool|
     -> Option<((f32, f32, f32), LightData)> {
        let rpos_raw = target_pos;
        let bpos = target_pos.to_board_position();
        let p = bpos.ndidx_checked(bf.map_size)?;
        let mut lux_fl = [0_f32; 3];
        let mut lightdata = LightData::default();
        for (flpos, fldir, flpower, flcolor, fltype, flvismap) in flashlights.iter() {
            let d2 = rpos_raw.distance2(flpos);
            let fl = if is_light_sensitive && d2 < 4.0 {
                // Smooth, distance-only lighting for players/ghosts near light sources
                // Using a 1/r falloff for proximity boost as requested.
                let dist = d2.sqrt();
                // flpower is already adjusted. The 2.0 factor provides a damped proximity boost.
                flpower / (dist + 5.0) * 2.0
            } else {
                let fldir = fldir.with_max_dist(200.0);
                let focus = (fldir.distance() + 0.1).max(6.0) / 20.0;
                let lpos = *flpos + fldir / (100.0 / focus + 20.0);
                let mut lpos = lpos.unrotate_by_dir(&fldir);
                let mut rpos = rpos_raw.unrotate_by_dir(&fldir);
                rpos.x -= lpos.x;
                rpos.y -= lpos.y;
                lpos.x = 0.0;
                lpos.y = 0.0;
                if rpos.x > 0.0 {
                    rpos.x = fastapprox::faster::pow(rpos.x, 1.0 / focus.clamp(1.0, 1.3));
                    rpos.y /= rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }
                if rpos.x < 0.0 {
                    rpos.x = -fastapprox::faster::pow(-rpos.x, (focus / 5.0 + 1.0).clamp(1.0, 4.0));
                    rpos.y *= -rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }

                let dist = (lpos.distance(&rpos) + 0.1)
                    .powf((fldir.distance() / 200.0).clamp(0.2, 1.0).recip());
                let flvis = flvismap[p];
                flpower / (dist + FL_MIN_DST)
                    * flvis.clamp(0.0001, 1.0)
                    * (focus + 0.5).clamp(0.5, 8.0)
            };
            let flsrgba = flcolor.to_srgba();
            lux_fl[0] += fl * flsrgba.red;
            lux_fl[1] += fl * flsrgba.green;
            lux_fl[2] += fl * flsrgba.blue;
            let ld = LightData::from_type(*fltype, fl);
            lightdata = lightdata.add(&ld);
        }
        let ambient_light = 0.0001 + tutorial_light_factor * 0.0001;
        lf.get(bpos.ndidx()).map(|lf| {
            let r = (lf.lux * lf.color.0 + lux_fl[0] + ambient_light) / exposure;
            let g = (lf.lux * lf.color.1 + lux_fl[1] + ambient_light) / exposure;
            let b = (lf.lux * lf.color.2 + lux_fl[2] + ambient_light) / exposure;

            // Artistic tonemapping: Sigmoid-ish curve with highlight protection (shoulder).
            let tonemap = |x: f32| {
                let v = x.powf(1.3);
                v * 1.8 / (1.0 + v * 0.45)
            };

            (
                (tonemap(r), tonemap(g), tonemap(b)),
                lightdata.add(&LightData::from_type(
                    LightType::Visible,
                    lf.lux + ambient_light,
                )),
            )
        })
    };

    let fpos_gamma = |target_pos: Position, o_light_sens: Option<&LightSensitive>| -> Option<f32> {
        let is_light_sensitive = o_light_sens.is_some();
        let gcolor = fpos_gamma_color(target_pos, is_light_sensitive);
        gcolor.map(|((r, g, b), _)| (r + g + b) / 3.0).map(|l| {
            let mut res = l;
            if let Some(ls) = o_light_sens {
                res += ls.bias;
                res = res.max(0.05);
            }
            // Reduced dynamic range: more aggressive highlight compression
            (res / 10.0).tanh() * 10.0
        })
    };

    let fpos_gamma_corner =
        |target_pos: Position, o_light_sens: Option<&LightSensitive>| -> Option<f32> {
            let x = target_pos.x;
            let y = target_pos.y;
            let z = target_pos.z.round() as i64;
            let x0 = x.floor() as i64;
            let y0 = y.floor() as i64;
            let x1 = x.ceil() as i64;
            let y1 = y.ceil() as i64;

            let mut total = 0.0;
            let mut count = 0.0;
            for tx in [x0, x1] {
                for ty in [y0, y1] {
                    let bpos = BoardPosition { x: tx, y: ty, z };
                    if let Some(p) = bpos.ndidx_checked(bf.map_size)
                        && vf.visibility_field[p] > 0.00001
                        && let Some(l) = fpos_gamma(bpos.to_position(), o_light_sens)
                    {
                        total += l;
                        count += 1.0;
                    }
                }
            }
            if count > 0.0 {
                Some(total / count)
            } else {
                fpos_gamma(target_pos, o_light_sens)
            }
        };

    // --- End of Shared Lighting Sampling Logic ---

    // let start = Instant::now();
    let materials1 = materials1.into_inner();

    let update_radius: usize = rng.random_range(8..32);
    let player_bpos = player_pos.to_board_position();
    let (map_width, map_height, map_depth) = bf.map_size;
    let player_ndidx = player_bpos.ndidx();
    let min_x = (player_ndidx.0).saturating_sub(update_radius);
    let max_x = (player_ndidx.0 + update_radius).min(map_width - 1);
    let min_y = (player_ndidx.1).saturating_sub(update_radius);
    let max_y = (player_ndidx.1 + update_radius).min(map_height - 1);
    let min_z = player_ndidx.2.saturating_sub(1);
    let max_z = (player_ndidx.2 + 1).min(map_depth - 1);
    let mut entities = Vec::with_capacity(256);

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let n = x + y * map_width + z * map_width * map_height;
                let dist = ((player_ndidx.0 as isize - x as isize).abs()
                    + (player_ndidx.1 as isize - y as isize).abs()
                    + (player_ndidx.2 as isize - z as isize).abs())
                    as usize;
                let min_threshold = ((n * BIG_PRIME) ^ mask) % VSMALL_PRIME;
                let skip_tile = min_threshold * dist / 9 > update_radius.saturating_sub(dist + 2);
                if vf.visibility_field[(x, y, z)] > 0.00001 {
                    if skip_tile {
                        for &entity in &bef.0[(x, y, z)] {
                            if let Ok(comp) = qt2.get(entity)
                                && (comp.4.is_none()
                                    || comp.7.is_some()
                                    || comp.8.is_some()
                                    || comp.10.0.is_some()
                                    || comp.10.5.is_some())
                            {
                                entities.push(entity);
                            }
                        }
                    } else {
                        entities.extend_from_slice(&bef.0[(x, y, z)]);
                    }
                }
            }
        }
    }

    for e in visible.iter() {
        if let Ok((
            _entity,
            _pos,
            _mat,
            _vis,
            o_behavior,
            _o_spectral_influence,
            o_interactive,
            o_ethereal,
            o_ecto_vis,
            _o_spectral_clarity,
            (o_light_sens, _o_ir_sens, _o_uv_sens, _o_map_color, _o_miasma, o_alpha_mod),
        )) = qt2.get(*e)
        {
            // Ensure entities with hover state changes are always processed
            // Also always process ghosts and breaches to ensure smooth oscillation
            // And light sensitive entities (player) to ensure smooth lighting
            if o_interactive.map(|x| x.hovered).unwrap_or_default()
                || o_behavior.is_none()
                || o_ethereal.is_some()
                || o_ecto_vis.is_some()
                || o_light_sens.is_some()
                || o_alpha_mod.is_some()
                || rng.random_range(0..100) < 15
            {
                entities.push(*e);
            }
        }
    }

    for entity in entities.iter() {
        let min_threshold: f32 = rng.random::<f32>() / 10.0;
        if let Ok((
            _entity_id,
            pos,
            mat,
            mut vis,
            o_behavior,
            o_spectral_influence,
            o_interactive,
            o_ethereal,
            o_ecto_vis,
            o_spectral_clarity,
            (o_light_sens, o_ir_sens, o_uv_sens, o_map_color, o_miasma, o_alpha_mod),
        )) = qt2.get_mut(*entity)
        {
            let on_hover = o_interactive
                .map(|x| x.hovered && mouse_visibility.is_visible)
                .unwrap_or_default();
            let mut opacity: f32 = 1.0;
            let mut exposure = exposure;
            if let Some(ls) = o_light_sens {
                exposure /= ls.exposure_factor;
            }

            let mut bpos = pos.to_board_position_size(bf.map_size);
            if let Some(behavior) = o_behavior {
                bpos.x += behavior.p.display.light_recv_offset.0;
                bpos.y += behavior.p.display.light_recv_offset.1;
                if behavior.p.display.auto_hide {
                    // Make big objects semitransparent when the player is behind them
                    const MAX_DIST: f32 = 8.0;
                    let dist = pos.distance(&player_pos);
                    if dist < MAX_DIST {
                        let delta_z = perspective::to_screen_coord(*pos).z
                            - perspective::to_screen_coord(player_pos).z;
                        if delta_z > 0.0 {
                            opacity = 0.1;
                        }
                    }
                }
            } else {
                let visibility: f32 = vf.visibility_field[bpos.ndidx()].clamp(0.0, 1.0);
                opacity = (visibility * 1.5).clamp(0.0001, 1.0);
            }

            // Use a margin (that should be baked on the map) to avoid negative access.
            if bpos.x < 2 || bpos.y < 2 {
                continue;
            }

            let mut lux_c = fpos_gamma(*pos, o_light_sens).unwrap_or(0.0);

            // Corner offsets correspond to tile intersections.
            // In our isometric perspective (PERSPECTIVE_X/Y in perspective.rs):
            // (+0.5, +0.5) -> Screen Right
            // (-0.5, -0.5) -> Screen Left
            // (-0.5, +0.5) -> Screen Top
            // (+0.5, -0.5) -> Screen Bottom
            let mut lux_right = fpos_gamma_corner(
                *pos + Direction {
                    dx: 0.5,
                    dy: 0.5,
                    dz: 0.0,
                },
                o_light_sens,
            )
            .unwrap_or(lux_c);
            let mut lux_left = fpos_gamma_corner(
                *pos + Direction {
                    dx: -0.5,
                    dy: -0.5,
                    dz: 0.0,
                },
                o_light_sens,
            )
            .unwrap_or(lux_c);
            let mut lux_top = fpos_gamma_corner(
                *pos + Direction {
                    dx: -0.5,
                    dy: 0.5,
                    dz: 0.0,
                },
                o_light_sens,
            )
            .unwrap_or(lux_c);
            let mut lux_bot = fpos_gamma_corner(
                *pos + Direction {
                    dx: 0.5,
                    dy: -0.5,
                    dz: 0.0,
                },
                o_light_sens,
            )
            .unwrap_or(lux_c);

            let ((mut r, mut g, mut b), light_data) =
                fpos_gamma_color(*pos, o_light_sens.is_some())
                    .unwrap_or(((1.0, 1.0, 1.0), LightData::UNIT_VISIBLE));
            if let Some(ls) = o_light_sens {
                r = (r + ls.bias).max(0.05);
                g = (g + ls.bias).max(0.05);
                b = (b + ls.bias).max(0.05);
            }
            let ld = light_data.normalize();

            let (att_charge, rep_charge) = o_spectral_influence
                .map(|x| match x.influence_type {
                    SpectralInfluenceType::Attractive => (x.charge_value.abs().sqrt() + 0.01, 0.0),
                    SpectralInfluenceType::Repulsive => (0.0, x.charge_value.abs().sqrt() + 0.01),
                })
                .unwrap_or_default();
            let rgbl = (r + g + b) / 3.0 + 1.0;
            g += light_data.ultraviolet * att_charge * 2.5 * rgbl;
            b += light_data.infrared * (att_charge + rep_charge) * 2.5 * rgbl;
            b += light_data.red * rep_charge * 0.01 * rgbl;
            r /= 1.0
                + light_data.red * rep_charge * 50.0 * rgbl
                + light_data.ultraviolet * att_charge * 12.0 * rgbl;
            g /= 1.0 + light_data.red * rep_charge * 10.0 * rgbl;
            b /= 1.0
                + light_data.infrared * (att_charge + rep_charge) * 10.0 * rgbl
                + light_data.ultraviolet * att_charge * 12.0 * rgbl;

            if let Some(behavior) = o_behavior
                && behavior.p.movement.walkable
            {
                lightdata_map.insert(bpos.clone(), light_data);
            }
            let max_color = r.max(g).max(b).max(0.2);
            let mut src_color_base = Color::srgb(r / max_color, g / max_color, b / max_color);
            let mut smooth_f: f32 = 0.3;
            let mut smooth_a: f32 = 1.0;

            if o_behavior.is_none() {
                // Players and characters need smoother color transitions than tiles
                smooth_f = 3.0;
            }
            let map_color = o_map_color.map(|x| x.color).unwrap_or_default();

            if let Some(am) = o_alpha_mod {
                opacity *= (am.frequency * elapsed).sin() * am.amplitude + (1.0 - am.amplitude);
            }

            if let Some(uv_sens) = o_uv_sens {
                opacity = (opacity + ld.ultraviolet * uv_sens.intensity).clamp(0.0, 1.3);
                let f = (ld.ultraviolet * uv_sens.color_shift).clamp(0.0, 1.0);
                src_color_base = lerp_color(src_color_base, css::MEDIUM_SLATE_BLUE.into(), f);
            }
            if let Some(ir_sens) = o_ir_sens {
                opacity = (opacity + ld.infrared * ir_sens.intensity).clamp(0.0, 1.3);
            }

            if let Some(ethereal) = o_ethereal {
                smooth_f = 299.0;
                smooth_a = 199.0;
                if ethereal.warning_active {
                    src_color_base = lerp_color(
                        css::RED.into(),
                        css::ALICE_BLUE.into(),
                        ethereal.warning_intensity.clamp(0.0, 1.0),
                    );
                } else if ethereal.hunt_target {
                    src_color_base = lerp_color(
                        css::RED.into(),
                        css::ALICE_BLUE.into(),
                        (ethereal.calm_time_secs / 10.0).clamp(0.0, 1.0),
                    );
                }
            }

            if let Some(_ecto) = o_ecto_vis {
                smooth_f = 99.0;
                smooth_a = 99.0;
            }

            let occlusion = o_behavior
                .map(|b| b.obsolete_occlusion_type())
                .unwrap_or(Orientation::None);
            match occlusion {
                Orientation::None => {}
                Orientation::XAxis => {
                    lux_top = lux_c;
                    lux_bot = lux_c;
                }
                Orientation::YAxis => {
                    lux_right = lux_c;
                    lux_left = lux_c;
                }
                Orientation::Both => {
                    lux_top = lux_c;
                    lux_bot = lux_c;
                    lux_right = lux_c;
                    lux_left = lux_c;
                }
            }
            opacity = opacity
                .min(vf.visibility_field[bpos.ndidx()] * 2.0)
                .clamp(0.0, 1.0);
            let mut new_mat = materials1.get(mat).unwrap().clone();
            let orig_mat = new_mat.clone();

            // remove brightness calculation for main tile:
            let mut dst_color = src_color_base;

            if let Some(ethereal) = o_ethereal
                && !ethereal.warning_active
                && !ethereal.hunt_target
            {
                let orig_opacity = opacity;

                // Make the ghost oscilate to increase visibility:
                let osc1 = (elapsed * 1.0 * difficulty.0.evidence_visibility).sin() * 0.25 + 0.75;
                let osc2 = (elapsed * 1.15 * difficulty.0.evidence_visibility).cos() * 0.5 + 0.5;
                opacity = opacity.min(osc1 + 0.2) / (1.0 + ethereal.warp / 5.0)
                    * difficulty.0.evidence_visibility;
                let l = (dst_color.luminance() + osc2) / 2.0;
                dst_color = dst_color.with_luminance(l);
                let r = dst_color.to_srgba().red;
                let g = dst_color.to_srgba().green;
                let clarity = o_spectral_clarity.cloned().unwrap_or_default();
                let e_uv = ld.ultraviolet * 13.0 * clarity.uv.max(0.0);
                let e_rl = (ld.red * 52.0 * clarity.rl.max(0.0)).clamp(0.0, 1.5);
                let e_infra = (ld.infrared * 1.1 * difficulty.0.evidence_visibility).sqrt();
                let f = (ld.visible * difficulty.0.evidence_visibility * 0.5 + ld.infrared * 4.0)
                    .clamp(0.001, 0.999);
                opacity = opacity * f + orig_opacity * (1.0 - f);
                opacity *= (clarity.alpha * 0.5
                    + 0.5
                    + e_uv
                    + e_rl
                    + ld.ultraviolet * 2.0
                    + ld.red * 10.0
                    + ld.infrared)
                    .clamp(difficulty.0.evidence_visibility * 0.1, 1.0);
                let srgba = dst_color
                    .with_luminance(
                        (l * ld.visible - ld.infrared - ethereal.hit_delta * 3.0).clamp(0.0, 1.0),
                    )
                    .to_srgba();

                let k_hit = (ethereal.hit_delta + ethereal.miss_delta)
                    .clamp(0.0, 1.0)
                    .cbrt();
                opacity = opacity * (1.0 - k_hit) + orig_opacity.cbrt() * k_hit;

                dst_color = srgba
                    .with_red(r * ld.visible + e_rl * 1.1 + ethereal.miss_delta / 2.0)
                    .with_green(g * ld.visible + e_uv + e_rl + ethereal.miss_delta / 2.5)
                    .into();
                dst_color = dst_color
                    .with_luminance((dst_color.luminance() - e_infra / 2.0).clamp(0.0, 1.0));
            }

            if let Some(ecto) = o_ecto_vis
                && ecto.use_breach_curve
            {
                let e_nv = ld.ultraviolet.cbrt() / 10.0 * difficulty.0.evidence_visibility
                    - ld.infrared * 3.0
                    + (difficulty.0.evidence_visibility / 7.0
                        + ld.visible * difficulty.0.evidence_visibility
                        - ld.infrared)
                        .powi(3);
                opacity *= ((dst_color.luminance() / 2.0) + e_nv / 4.0).clamp(0.0, 0.9);
                opacity = opacity.min(vf.visibility_field[bpos.ndidx()]).cbrt();
                let l = dst_color.luminance();
                let rnd_f = rng.random_range(-1.0..1.0_f32).powi(3);
                // Make the breach oscilate to increase visibility:
                let osc1 = ((elapsed * 0.92).sin() * 10.0 + 8.0).tanh() * 0.5 + 0.5;

                dst_color = dst_color.with_luminance(
                    ((l * (ld.visible - ld.infrared) + e_nv) * osc1).clamp(0.0, 0.99),
                );
                let lin_dst_color = dst_color.to_linear();

                dst_color = lin_dst_color
                    .with_green(
                        lin_dst_color.green
                            + ld.ultraviolet
                                * difficulty.0.evidence_visibility
                                * (1.3 - osc1 + rnd_f / 14.0),
                    )
                    .with_red(
                        lin_dst_color.red
                            + ld.ultraviolet
                                * difficulty.0.evidence_visibility
                                * 1.2
                                * (1.4 - osc1 + rnd_f / 24.0),
                    )
                    .into();
            }

            if let Some(_miasma_sprite) = o_miasma {
                let mut total_pressure = 0.0_f32;
                let bpos_m = pos.to_board_position();
                for npos in bpos_m.iter_xy_neighbors(1, bf.map_size) {
                    total_pressure += miasma.pressure_field[npos.ndidx()];
                }
                total_pressure /= 10.0;
                opacity *= (1.0_f32 - total_pressure).clamp(0.0_f32, 1.0_f32);
            }

            let opacity = opacity.clamp(0.000, 1.3);
            const A_DELTA: f32 = 0.02;

            let prev_a = new_mat.data.color.alpha();
            let f_a = 1.0 / (1.0 + smooth_a);

            let next_a = opacity * f_a + prev_a * (1.0 - f_a);
            let mut new_a = if (next_a - opacity).abs() < A_DELTA {
                opacity
            } else {
                next_a - A_DELTA * (next_a - opacity).signum()
            };
            if o_behavior.is_none() {
                // For special entities, avoid linear stepping and just use the exponential lerp
                new_a = opacity;
            }

            new_a = (new_a * map_color.alpha()).clamp(0.0, 1.0);
            dst_color.set_alpha(new_a);

            // Sound field visualization:
            let f_gamma = |lux: f32| fastapprox::faster::pow(lux, 0.9);
            const K_COLD: f32 = 0.6;
            let cold_f = (1.0 - (lux_c / K_COLD).tanh()) * 2.0;
            const DARK_COLOR: Color = Color::srgba(0.247 / 1.5, 0.714 / 1.5, 0.878, 1.0);
            const DARK_COLOR2: Color = Color::srgba(0.2, 0.6, 1.0, 1.0);
            let dark_color2 = DARK_COLOR2;
            let exp_color =
                ((-(exposure + 0.0001).ln() / 2.0 - 1.5 + cold_f).tanh() + 0.5).clamp(0.0, 1.0);

            // Apply tutorial boost to the blue hue and ensure it never goes 100% dark.
            // Darker missions (Standard+) will have 0.0 boost.
            let exp_color_tint = (exp_color + tutorial_light_factor * 0.20).clamp(0.0, 1.0);
            let dark = lerp_color(
                Color::BLACK,
                DARK_COLOR,
                (exp_color_tint / 16.0 + tutorial_light_factor * 0.01).clamp(0.0, 1.0),
            );

            let dark2 = lerp_color(
                Color::WHITE,
                dark_color2,
                exp_color_tint / f_gamma(lux_c).clamp(1.0, 300.0),
            );
            if o_behavior.is_some() {
                new_mat.data.ambient_color = dark.with_alpha(0.0).into();
            } else {
                new_mat.data.ambient_color = Color::NONE.into();
            }

            // Convert both colors to LinearRgba for multiplication
            let linear_dst_color = LinearRgba::from(dst_color);
            let linear_dark2_color = LinearRgba::from(dark2);

            // Perform the multiplication in the LinearRgba space
            let new_color = linear_dst_color.to_vec4() * linear_dark2_color.to_vec4();

            // Convert back to Color
            let new_color = LinearRgba::from_vec4(new_color);

            let src_color_old = new_mat.data.color;
            if o_behavior.is_none() {
                // For special entities, use the exponential decay lerp for color/alpha
                let f_c = 1.0 / (1.0 + smooth_f);
                new_mat.data.color = LinearRgba::from_vec4(
                    (src_color_old.to_vec4() * (1.0 - f_c) + new_color.to_vec4() * f_c)
                        .clamp(Vec4::ZERO, Vec4::ONE),
                );
            } else {
                // For regular tiles, color snaps to the new lighting
                new_mat.data.color = new_color;
            }
            // new_mat.data.color = Srgba::rgb(1.0, 1.0, 1.0).into(); // --- debug for no color but gamma

            const BRIGHTNESS: f32 = 1.0;
            let tint_comp = (1.0 - src_color_base.luminance()).clamp(0.0, 1.0);
            let smooth_f = prev_a + 0.3 + smooth_f;
            let gamma_mean = |a: f32, b: f32| {
                (a * smooth_f
                    + f_gamma(
                        b * BRIGHTNESS * (1.0 + cold_f + (exp_color * 2.0).powi(2))
                            + (tint_comp + cold_f * 2.0 + (exp_color * 2.0).powi(2))
                                / (10.0 + exposure + b),
                    )
                    + exp_color / 40.0)
                    / (1.0 + smooth_f)
            };
            // let gamma_mean = |_a: f32, _b: f32| 1.0; // --- debug for color but no gamma.
            // Mapping to vertices:
            // gtl (Top): Logic (0.5, -0.5)
            // gtr (Right): Logic (0.5, 0.5)
            // gbl (Left): Logic (-0.5, -0.5)
            // gbr (Bottom): Logic (-0.5, 0.5)
            new_mat.data.gamma = gamma_mean(new_mat.data.gamma, lux_c);
            new_mat.data.gtl = gamma_mean(new_mat.data.gtl, lux_top);
            new_mat.data.gtr = gamma_mean(new_mat.data.gtr, lux_right);
            new_mat.data.gbl = gamma_mean(new_mat.data.gbl, lux_left);
            new_mat.data.gbr = gamma_mean(new_mat.data.gbr, lux_bot);

            if on_hover {
                lux_c += 1.0;
                new_mat.data.ambient_color = Color::srgb(0.20, 0.20, 0.0).into();
                new_mat.data.color = Color::srgb(
                    (new_color.red + 0.5).min(1.0),
                    (new_color.green + 0.5).min(1.0),
                    new_color.blue * 0.3,
                )
                .into();
                // We update gamma with the hover boost too
                new_mat.data.gamma = gamma_mean(new_mat.data.gamma, lux_c);
            }

            const DEBUG_LIGHTING: bool = false;
            if DEBUG_LIGHTING
                && bpos == player_bpos
                && (time.elapsed_secs() % 1.0) < time.delta_secs()
            {
                let f_g = f_gamma(lux_c);
                info!(
                    "Adapt: rl:{:.4} al:{:.4} exp:{:.4} mc:{:.4} lc:{:.4} fg:{:.4} ec:{:.2} cf:{:.2} g:{:.2} c:{:?}",
                    raw_lux,
                    lg.exposure_lux,
                    exposure,
                    max_color,
                    lux_c,
                    f_g,
                    exp_color,
                    cold_f,
                    new_mat.data.gamma,
                    new_mat.data.color
                );
            }

            const DEBUG_SOUND: bool = false;
            if DEBUG_SOUND && let Some(sf) = sg.sound_field.get(&bpos) {
                let l: f32 = sf.iter().map(|x: &Vec2| x.length() + 0.01).sum();
                if l > 0.0001 {
                    new_mat.data.gamma = 2.0;
                    new_mat.data.color = Color::srgb(1.0, l / 4.0, l / 16.0).into();
                }
            }
            const DEBUG_TEMPERATURE: bool = false;
            if DEBUG_TEMPERATURE {
                let temp_celsius = kelvin_to_celsius(tg.temperature_field[bpos.ndidx()]);
                // Map temperature to continuous color gradient: 0°C to 16°C -> blue-cyan-green-yellow-red
                let normalized_temp = (temp_celsius / 16.0).clamp(0.0, 1.0);

                // Create smooth color transition using HSV-like interpolation
                let (r, g, b) = if normalized_temp <= 0.25 {
                    // Blue to Cyan (0.0 to 0.25)
                    let t = normalized_temp / 0.25;
                    (0.0, t, 1.0)
                } else if normalized_temp <= 0.5 {
                    // Cyan to Green (0.25 to 0.5)
                    let t = (normalized_temp - 0.25) / 0.25;
                    (0.0, 1.0, 1.0 - t)
                } else if normalized_temp <= 0.75 {
                    // Green to Yellow (0.5 to 0.75)
                    let t = (normalized_temp - 0.5) / 0.25;
                    (t, 1.0, 0.0)
                } else {
                    // Yellow to Red (0.75 to 1.0)
                    let t = (normalized_temp - 0.75) / 0.25;
                    (1.0, 1.0 - t, 0.0)
                };
                new_mat.data.ambient_color = Color::srgb(r / 3.0, g / 3.0, b / 3.0).into();
                new_mat.data.gamma = 2.5;
                new_mat.data.color = Color::srgba(r, g, b, new_mat.data.color.alpha()).into();
            }
            let invisible = new_mat.data.color.alpha() < 0.005
                || o_behavior.map(|b| b.p.display.disable).unwrap_or_default();
            let new_vis = if invisible {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            if new_vis != *vis {
                if invisible {
                    visible.remove(entity);
                } else {
                    visible.insert(entity.to_owned());
                }
                *vis = new_vis;
            } else if !invisible && !visible.contains(entity) {
                visible.insert(entity.to_owned());
            }
            let delta = orig_mat.data.delta(&new_mat.data);
            let thr = if IS_WASM { 0.2 } else { 0.02 };
            let auto_hide = o_behavior
                .map(|b| b.p.display.auto_hide)
                .unwrap_or_default();
            if auto_hide
                || delta > thr + min_threshold
                || o_ethereal.is_some()
                || o_ecto_vis.is_some()
                || o_behavior.is_none()
            {
                let mat = materials1.get_mut(mat).unwrap();
                mat.data = new_mat.data;
                // change_count += 1;
            }
        }
    }

    // Light ilumination for sprites on map that aren't part of the map (player,
    // ghost, ghost breach)
    for (
        _entity,
        pos,
        mut o_sprite,
        o_mat,
        o_light_sens,
        _o_uv_sens,
        o_ir_sens,
        _o_ecto_vis,
        _o_ethereal,
        _o_luminescent,
        o_shadow_caster,
        o_color,
        uv_reactive,
        o_miasma,
        _o_spectral_clarity,
    ) in qt.iter_mut()
    {
        let sprite_color = if let Some(sprite) = o_sprite.as_ref() {
            sprite.color
        } else if let Some(mat_handle) = o_mat {
            materials1
                .get(mat_handle)
                .map(|m| Color::from(m.data.color))
                .unwrap_or(Color::WHITE)
        } else {
            continue;
        };
        let bpos = pos.to_board_position_size(bf.map_size);
        let map_color = o_color.map(|x| x.color).unwrap_or_default();
        let visibility: f32 = vf.visibility_field[bpos.ndidx()].clamp(0.0, 1.0);
        let mut opacity: f32 = map_color.alpha() * visibility;
        opacity = (opacity.powf(0.5) * 2.0 - 0.1).clamp(0.0001, 1.0);

        let (gcolor, ld_abs) = fpos_gamma_color(*pos, o_light_sens.is_some())
            .unwrap_or(((1.0, 1.0, 1.0), LightData::UNIT_VISIBLE));

        let ld = ld_abs.normalize();

        if o_light_sens.is_none() && opacity > 0.0001 {
            // Note: Previously checked light_sz < 3.0, but now we always have data.
        }
        let mut src_color = map_color.with_alpha(1.0);
        let uv_reactive = uv_reactive.map(|x| x.0).unwrap_or_default();
        src_color = lerp_color(
            src_color,
            css::GREEN.into(),
            (ld.ultraviolet * uv_reactive).sqrt(),
        );
        let mut dst_color = {
            let r: f32 = (bpos.mini_hash() - 0.4) / 50.0;
            let mut rel_lux = (gcolor.0 + gcolor.1 + gcolor.2) / 3.0;

            rel_lux += ld.ultraviolet * uv_reactive * 5.0;

            if let Some(light_sens) = o_light_sens {
                rel_lux = (rel_lux + light_sens.bias).max(0.05);
            }
            compute_color_exposure(rel_lux, r, 1.0, src_color)
        };

        // 20.0;
        let mut smooth: f32 = 1.0;
        if let Some(ir_sens) = o_ir_sens
            && let Some(threshold) = ir_sens.thresholds
        {
            smooth = 10.0;
            let total_light =
                ld_abs.visible + ld_abs.red + ld_abs.ultraviolet + ld_abs.infrared + 0.1;
            let ir_ratio = ld_abs.infrared / total_light;
            if ir_ratio > threshold && ld_abs.infrared > 0.1 && ld_abs.visible < 0.5 {
                opacity = (ir_ratio * 2.0 - 1.0).powi(2) * ld_abs.infrared.sqrt() * visibility;
                opacity = (opacity * ir_sens.intensity).clamp(0.0, 1.0);
            } else {
                opacity = 0.0;
            }
        }

        let old_a = (sprite_color.alpha()).clamp(0.0001, 1.0);
        if let Some(miasma_sprite) = o_miasma {
            let bpos = pos.to_board_position();
            let mut total_pressure = 0.0;
            let mut total_weight = 0.0;

            for dx in -1..=1 {
                for dy in -1..=1 {
                    let neighbor_pos = BoardPosition {
                        x: bpos.x + dx,
                        y: bpos.y + dy,
                        z: bpos.z,
                    };

                    if let Some(neighbor_pressure) = miasma.pressure_field.get(neighbor_pos.ndidx())
                    {
                        // Calculate distance from sprite's *actual* position to the
                        // *center* of the neighbor tile. This is important for smooth
                        // weighting.
                        let neighbor_center = neighbor_pos.to_position_center();
                        let distance = pos.distance(&neighbor_center); // Euclidean distance
                        let weight = (distance + 0.1).recip(); // Avoid division by zero

                        total_pressure += neighbor_pressure * weight;
                        total_weight += weight;
                    }
                }
            }

            let average_pressure = if total_weight > 0.0 {
                total_pressure / total_weight
            } else {
                0.0 // Default to 0 if no neighbors have pressure (shouldn't happen)
            };

            let miasma_visibility = average_pressure.max(0.0).sqrt()
                * miasma_config.miasma_visibility_factor
                * miasma_sprite.life.clamp(0.0, 1.0)
                * (ld.magnitude().atan() / 1.1 + 0.4);

            dst_color = dst_color
                .with_luminance((dst_color.luminance().sqrt() * 0.9 + 0.01).clamp(0.0, 1.0));
            opacity = opacity.max(0.0);
            opacity *= miasma_visibility.clamp(0.0, 0.8)
                * miasma_sprite.visibility
                * (dst_color.luminance().sqrt() * 0.8 + 0.2);
        }
        dst_color.set_alpha(
            ((opacity + old_a * smooth) / (smooth + 1.0)).clamp(0.0, 1.0) * map_color.alpha(),
        );
        let src_linear = sprite_color.to_linear();
        let dst_linear = dst_color.to_linear();
        let f = if o_shadow_caster.is_some() {
            0.01
        } else {
            0.11
        }; // Smoothing factor
        let smooth_color = LinearRgba::from_vec4(
            (src_linear.to_vec4() * (1.0 - f) + dst_linear.to_vec4() * f)
                .clamp(Vec4::ZERO, Vec4::ONE),
        );
        if let Some(sprite) = o_sprite.as_mut() {
            sprite.color = smooth_color.into();
        } else if let Some(mat_handle) = o_mat
            && let Some(mat) = materials1.get_mut(mat_handle)
        {
            mat.data.color = smooth_color;
        }
    }
    for (bpos, ld) in lightdata_map.into_iter() {
        lg.light_field[bpos.ndidx()].additional = ld;
    }

    measure.end_ms();
}

pub(crate) fn app_setup(_app: &mut App) {}
