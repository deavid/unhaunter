use bevy::prelude::*;
use bevy::utils::Instant;
use fastapprox::faster;

use uncore::behavior::Behavior;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::position::Position;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::resources::roomdb::RoomDB;
use uncore::resources::visibility_data::VisibilityData;
use uncore::types::board::cached_board_pos::CachedBoardPos;
use uncore::types::board::light_field_sector::LightFieldSector;
use uncore::{
    resources::board_data::BoardData,
    types::board::fielddata::{CollisionFieldData, LightFieldData},
};

use crate::board::spritedb::SpriteDB;

/// Main system of board that moves the tiles to their correct place in the screen
/// following the isometric perspective.
pub fn apply_perspective(mut q: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = pos.to_screen_coord();
    }
}

pub struct UnhaunterBoardPlugin;

impl Plugin for UnhaunterBoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardData>()
            .init_resource::<VisibilityData>()
            .init_resource::<SpriteDB>()
            .init_resource::<RoomDB>()
            .add_systems(Update, apply_perspective)
            .add_systems(PostUpdate, boardfield_update)
            .add_event::<BoardDataToRebuild>();
    }
}

pub fn rebuild_collision_data(bf: &mut ResMut<BoardData>, qt: &Query<(&Position, &Behavior)>) {
    // info!("Collision rebuild");
    assert_eq!(
        bf.collision_field.shape(),
        [bf.map_size.0, bf.map_size.1, bf.map_size.2]
    );
    bf.collision_field.fill(CollisionFieldData::default());

    for (pos, _behavior) in qt.iter().filter(|(_p, b)| b.p.movement.walkable) {
        let bpos = pos.to_board_position();
        let colfd = CollisionFieldData {
            player_free: true,
            ghost_free: true,
            see_through: false,
        };
        bf.collision_field[bpos.ndidx()] = colfd;
    }
    for (pos, behavior) in qt.iter().filter(|(_p, b)| b.p.movement.player_collision) {
        let bpos = pos.to_board_position();
        let colfd = CollisionFieldData {
            player_free: false,
            ghost_free: !behavior.p.movement.ghost_collision,
            see_through: behavior.p.light.see_through,
        };
        bf.collision_field[bpos.ndidx()] = colfd;
    }
}

pub fn boardfield_update(
    mut bf: ResMut<BoardData>,
    mut ev_bdr: EventReader<BoardDataToRebuild>,
    qt: Query<(&Position, &Behavior)>,
) {
    if ev_bdr.is_empty() {
        return;
    }

    // Here we will recreate the field (if needed? - not sure how to detect that) ...
    // maybe add a timer since last update.
    let mut bdr = BoardDataToRebuild::default();

    // Merge all the incoming events into a single one.
    for b in ev_bdr.read() {
        if b.collision {
            bdr.collision = true;
        }
        if b.lighting {
            bdr.lighting = true;
        }
    }

    if bdr.collision {
        rebuild_collision_data(&mut bf, &qt);
    }

    if bdr.lighting {
        // Rebuild lighting field since it has changed info!("Lighting rebuild");
        let build_start_time = Instant::now();
        let cbp = CachedBoardPos::new();
        bf.exposure_lux = 1.0;
        bf.light_field.fill(LightFieldData::default());

        // Dividing by 4 so later we don't get an overflow if there's no map.
        let first_p = qt
            .iter()
            .next()
            .map(|(p, _b)| p.to_board_position())
            .unwrap_or_default();
        let mut min_x = first_p.x;
        let mut min_y = first_p.y;
        let mut min_z = first_p.z;
        let mut max_x = first_p.x;
        let mut max_y = first_p.y;
        let mut max_z = first_p.z;
        for (pos, behavior) in qt.iter() {
            let pos = pos.to_board_position();
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            min_z = min_z.min(pos.z);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
            max_z = max_z.max(pos.z);
            let src = &bf.light_field[pos.ndidx()];
            bf.light_field[pos.ndidx()] = LightFieldData {
                lux: behavior.p.light.emmisivity_lumens() + src.lux,
                transmissivity: behavior.p.light.transmissivity_factor() * src.transmissivity
                    + 0.0001,
                additional: src.additional.add(&behavior.p.light.additional_data()),
            };
        }

        // info!( "Collecting time: {:?} - sz: {}", build_start_time.elapsed(),
        // bf.light_field.len() );
        let mut lfs = LightFieldSector::new(min_x, min_y, min_z, max_x, max_y, max_z);
        for (k, v) in bf.light_field.indexed_iter() {
            lfs.insert(k.0 as i64, k.1 as i64, k.2 as i64, v.clone());
        }

        // let mut lfs_clone_time_total = Duration::ZERO; let mut shadows_time_total =
        // Duration::ZERO; let mut store_lfs_time_total = Duration::ZERO;
        for step in 0..3 {
            // let lfs_clone_time = Instant::now();
            let src_lfs = lfs.clone();

            // lfs_clone_time_total += lfs_clone_time.elapsed();
            let size = match step {
                0 => 26,
                1 => 8,
                2 => 6,
                3 => 3,
                _ => 6,
            };
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    for z in min_z..=max_z {
                        let Some(src) = src_lfs.get(x, y, z) else {
                            continue;
                        };

                        // if src.transmissivity < 0.5 && step > 0 && size > 1 { // Reduce light spread
                        // through walls // FIXME: If the light is on the wall, this breaks (and this is
                        // possible since the wall is really 1/3rd of the tile) continue; }
                        let root_pos = BoardPosition { x, y, z };
                        let mut src_lux = src.lux;
                        let min_lux = match step {
                            0 => 0.001,
                            1 => 0.000001,
                            _ => 0.0000000001,
                        };
                        let max_lux = match step {
                            0 => f32::MAX,
                            1 => 10000.0,
                            2 => 1000.0,
                            3 => 0.1,
                            _ => 0.01,
                        };
                        if src_lux < min_lux {
                            continue;
                        }
                        if src_lux > max_lux {
                            continue;
                        }

                        // Optimize next steps by only looking to harsh differences.
                        let nbors =
                            root_pos.iter_xy_neighbors_clamped(1, (min_x, min_y), (max_x, max_y));

                        if step > 0 {
                            let ldata_iter = nbors.filter_map(|b| {
                                lfs.get_pos(&b).map(|l| {
                                    (
                                        ordered_float::OrderedFloat(l.lux),
                                        ordered_float::OrderedFloat(l.transmissivity),
                                    )
                                })
                            });
                            let mut min_lux = ordered_float::OrderedFloat(f32::MAX);
                            let mut min_trans = ordered_float::OrderedFloat(2.0);
                            for (lux, trans) in ldata_iter {
                                min_lux = min_lux.min(lux);
                                min_trans = min_trans.min(trans);
                            }

                            // For smoothing steps only:
                            if *min_trans > 0.7 && src_lux / (*min_lux + 0.0001) < 1.9 {
                                // If there are no walls nearby, we don't reflect light.
                                continue;
                            }
                        }

                        // This controls how harsh is the light
                        if step > 0 {
                            src_lux /= 5.5;
                        } else {
                            src_lux /= 1.01;
                        }

                        // let shadows_time = Instant::now(); This takes time to process:
                        let nbors = root_pos.iter_xy_neighbors_clamped(
                            size,
                            (min_x, min_y),
                            (max_x, max_y),
                        );

                        // reset the light value for this light, so we don't count double.
                        lfs.get_mut_pos(&root_pos).unwrap().lux -= src_lux;
                        let mut shadow_dist = [(size + 1) as f32; CachedBoardPos::TAU_I];

                        // Compute shadows
                        for pillar_pos in nbors.clone() {
                            // 60% of the time spent in compute shadows is obtaining this:
                            let Some(lf) = lfs.get_pos(&pillar_pos) else {
                                continue;
                            };

                            // let lf = unsafe { lfs.get_pos_unchecked(pillar_pos) }; t_x += lf.lux; continue;
                            let consider_opaque = lf.transmissivity < 0.5;
                            if !consider_opaque {
                                continue;
                            }
                            let min_dist = cbp.bpos_dist(&root_pos, &pillar_pos);
                            let angle = cbp.bpos_angle(&root_pos, &pillar_pos);
                            let angle_range = cbp.bpos_angle_range(&root_pos, &pillar_pos);
                            for d in angle_range.0..=angle_range.1 {
                                let ang = (angle as i64 + d)
                                    .rem_euclid(CachedBoardPos::TAU_I as i64)
                                    as usize;
                                shadow_dist[ang] = shadow_dist[ang].min(min_dist);
                            }
                        }

                        // shadows_time_total += shadows_time.elapsed(); FIXME: Possibly we want to smooth
                        // shadow_dist here - a convolution with a gaussian or similar where we preserve
                        // the high values but smooth the transition to low ones.
                        if src.transmissivity < 0.5 {
                            // Reduce light spread through walls
                            shadow_dist.iter_mut().for_each(|x| *x = 0.0);
                        }

                        // let size = shadow_dist .iter() .map(|d| (d + 1.5).round() as u32) .max()
                        // .unwrap() .min(size); let nbors = root_pos.xy_neighbors(size);
                        let light_height = 4.0;

                        // let mut total_lux = 0.1; for neighbor in nbors.iter() { let dist =
                        // cbp.bpos_dist(&root_pos, neighbor); let dist2 = dist + light_height; let angle
                        // = cbp.bpos_angle(&root_pos, neighbor); let sd = shadow_dist[angle]; let f =
                        // (faster::tanh(sd - dist - 0.5) + 1.0) / 2.0; total_lux += f / dist2 / dist2; }
                        // let store_lfs_time = Instant::now();
                        let total_lux = 2.0;

                        // new shadow method
                        for neighbor in nbors {
                            let dist = cbp.bpos_dist(&root_pos, &neighbor);

                            // let dist = root_pos.fast_distance_xy(neighbor);
                            let dist2 = dist + light_height;
                            let angle = cbp.bpos_angle(&root_pos, &neighbor);
                            let sd = shadow_dist[angle];
                            let lux_add = src_lux / dist2 / dist2 / total_lux;
                            if dist - 3.0 < sd {
                                // FIXME: f here controls the bleed through walls.
                                if let Some(lf) = lfs.get_mut_pos(&neighbor) {
                                    // 0.5 is too low, it creates un-evenness.
                                    const BLEED_TILES: f32 = 0.8;
                                    let f = (faster::tanh((sd - dist - 0.5) * BLEED_TILES.recip())
                                        + 1.0)
                                        / 2.0;

                                    // let f = 1.0;
                                    lf.lux += lux_add * f;
                                }
                            }
                        }
                        // store_lfs_time_total += store_lfs_time.elapsed();
                    }
                }
            }
            // info!( "Light step {}: {:?}; per size: {:?}", step, step_time.elapsed(),
            // step_time.elapsed() / size );
        }
        for (k, v) in bf.light_field.indexed_iter_mut() {
            let bpos = BoardPosition::from_ndidx(k);
            v.lux = lfs.get_pos(&bpos).unwrap().lux;
        }

        // let's get an average of lux values
        let mut total_lux = 0.0;
        for v in bf.light_field.iter() {
            total_lux += v.lux;
        }
        let avg_lux = total_lux / bf.light_field.len() as f32;
        bf.exposure_lux = (avg_lux + 2.0) / 2.0;

        // dbg!(lfs_clone_time_total); dbg!(shadows_time_total);
        // dbg!(store_lfs_time_total);
        info!(
            "Lighting rebuild - complete: {:?}",
            build_start_time.elapsed()
        );
    }
}
