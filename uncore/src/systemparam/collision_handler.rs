use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::board::boardposition::BoardPosition;
use crate::components::board::position::Position;
use crate::resources::board_data::BoardData;

/// System parameter for handling player collisions with the environment.
#[derive(SystemParam)]
pub struct CollisionHandler<'w> {
    /// Access to the game's board data, including collision information.
    bf: Res<'w, BoardData>,
}

impl CollisionHandler<'_> {
    const ENABLE_COLLISION: bool = true;
    const PILLAR_SZ: f32 = 0.3;
    const PLAYER_SZ: f32 = 0.5;
    // Threshold to determine when a position is considered "between floors"
    const FLOOR_TRANSITION_THRESHOLD: f32 = 0.49;

    pub fn delta(&self, pos: &Position) -> Vec3 {
        let mut delta = Vec3::ZERO;

        // Get the default board position (rounded to nearest integer)
        let bpos = pos.to_board_position();

        // Check if we're in a transitional z state between floors
        // If z is not close to an integer value (within threshold), we should check both floors
        let z_fractional = pos.z.fract().abs();
        let between_floors = z_fractional > Self::FLOOR_TRANSITION_THRESHOLD
            && z_fractional < (1.0 - Self::FLOOR_TRANSITION_THRESHOLD);

        // First check collisions on the current floor
        delta += self.check_floor_collisions(&bpos, pos);

        // If between floors, also check the adjacent floor
        if between_floors {
            let adjacent_z = if pos.z.fract() > 0.0 {
                bpos.z + 1 // Moving up to next floor
            } else {
                bpos.z - 1 // Moving down to previous floor
            };

            // Create a board position for the adjacent floor
            let adjacent_bpos = BoardPosition {
                x: bpos.x,
                y: bpos.y,
                z: adjacent_z,
            };

            // Check collisions on the adjacent floor
            delta += self.check_floor_collisions(&adjacent_bpos, pos);
        }

        delta
    }

    // Helper method to check collisions on a specific floor
    fn check_floor_collisions(&self, bpos: &BoardPosition, pos: &Position) -> Vec3 {
        let mut delta = Vec3::ZERO;

        for npos in bpos.iter_xy_neighbors_nosize(1) {
            let cf = self
                .bf
                .collision_field
                .get(npos.ndidx())
                .copied()
                .unwrap_or_default();

            if !cf.player_free && Self::ENABLE_COLLISION {
                let dpos = npos.to_position().to_vec3() - pos.to_vec3();
                if dpos.length() < Self::PILLAR_SZ + Self::PLAYER_SZ / 2.0 {
                    delta += dpos.normalize_or_zero() * 0.1;
                    warn!("DPOS LEN: {:?}", dpos);
                }
                let mut dapos = dpos.abs();
                dapos.z = 0.0;
                dapos.x -= Self::PILLAR_SZ;
                dapos.y -= Self::PILLAR_SZ;
                dapos.x = dapos.x.max(0.0);
                dapos.y = dapos.y.max(0.0);
                let ddist = dapos.distance(Vec3::ZERO);
                if ddist < Self::PLAYER_SZ {
                    if dpos.x < 0.0 {
                        dapos.x *= -1.0;
                    }
                    if dpos.y < 0.0 {
                        dapos.y *= -1.0;
                    }
                    let fix_dist = (Self::PLAYER_SZ - ddist).powi(2);
                    let dpos_fix = dapos / (ddist + 0.000001) * fix_dist;
                    delta += dpos_fix;
                }
            }
        }

        delta
    }
}
