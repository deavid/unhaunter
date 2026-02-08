use ndarray::Array3;
use std::collections::VecDeque;
use unbehavior::roomdb::RoomDB;
use unboard_core::types::fielddata::CollisionFieldData;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

pub(crate) fn compute_visibility(
    vis_field: &mut Array3<f32>,
    collision_field: &Array3<CollisionFieldData>,
    pos_start: &Position,
    roomdb: Option<&mut RoomDB>,
    pre_fill: bool,
) {
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
            let threshold = 2.0;
            let f = if npds < threshold {
                1.0
            } else {
                ((npds - pds) / npref).clamp(0.0, 1.0).powf(1.0)
            };
            let mut dst_f = src_f * f;
            if dst_f < 0.00001 {
                continue;
            }
            let k = if let Some(roomdb) = roomdb.as_ref() {
                match roomdb.room_tiles.get(&npos).is_some() {
                    // Decrease view range inside the location
                    true => 7.0,
                    false => 8.0,
                }
            } else {
                // For deployed gear
                7.0
            };
            dst_f /= 1.0 + ((npds - threshold) / k).clamp(0.0, 6.0);
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
}
