use std::ops::Range;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{direction::Direction, position::Position};

#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct BoardPosition {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl BoardPosition {
    // Tile sizes: Tiles are supposed to be divisible in a subgrid of 3x3x11,
    // where each sub-cube is 20cm x 20cm x 20cm.
    // Therefore, a single tile measures 60xm x 60cm x 220cm

    /// Size of the tile in X direction: 60cm
    pub fn size_x_meters() -> f32 {
        0.60
    }

    /// Size of the tile in Y direction: 60cm
    pub fn size_y_meters() -> f32 {
        0.60
    }

    /// Size of the tile in Z direction: 220cm
    pub fn size_z_meters() -> f32 {
        2.20
    }

    /// Area of a tile: 0.36m² - 3600cm²
    pub fn area_per_tile_m2() -> f32 {
        Self::size_x_meters() * Self::size_y_meters()
    }

    /// Volume of a tile: 0.216 m³ - 216 Liters
    pub fn volume_per_tile_m3() -> f32 {
        Self::size_x_meters() * Self::size_y_meters() * Self::size_z_meters()
    }

    pub fn from_ndidx(pos: (usize, usize, usize)) -> Self {
        Self {
            x: pos.0 as i64,
            y: pos.1 as i64,
            z: pos.2 as i64,
        }
    }
    pub fn ndidx(&self) -> (usize, usize, usize) {
        (self.x as usize, self.y as usize, self.z as usize)
    }

    pub fn ndidx_checked(&self, map_size: (usize, usize, usize)) -> Option<(usize, usize, usize)> {
        if self.x < 0
            || self.x >= map_size.0 as i64
            || self.y < 0
            || self.y >= map_size.1 as i64
            || self.z < 0
            || self.z >= map_size.2 as i64
        {
            None
        } else {
            Some((self.x as usize, self.y as usize, self.z as usize))
        }
    }

    pub fn ndidx_checked_margin(
        &self,
        map_size: (usize, usize, usize),
    ) -> Option<(usize, usize, usize)> {
        const MARGIN: i64 = 2;
        if self.x < MARGIN
            || self.x >= map_size.0 as i64 - MARGIN
            || self.y < MARGIN
            || self.y >= map_size.1 as i64 - MARGIN
            || self.z < 0
            || self.z >= map_size.2 as i64
        {
            None
        } else {
            Some((self.x as usize, self.y as usize, self.z as usize))
        }
    }

    pub fn to_position(&self) -> Position {
        Position {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
            global_z: 0.0,
        }
    }

    /// DEPRECATED: This is wrong. The center of the tile is BoardPosition::to_position(). This actually gives you the bottom right edge.
    pub fn to_position_center(&self) -> Position {
        Position {
            x: self.x as f32 + 0.5,
            y: self.y as f32 + 0.5,
            z: self.z as f32,
            global_z: 0.0,
        }
    }

    pub fn left(&self) -> Self {
        Self {
            x: (self.x - 1).max(0),
            y: self.y,
            z: self.z,
        }
    }

    pub fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn top(&self) -> Self {
        Self {
            x: self.x,
            y: (self.y - 1).max(0),
            z: self.z,
        }
    }

    pub fn bottom(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn _xy_neighbors_buf(&self, dist: u32, out: &mut Vec<BoardPosition>) {
        out.clear();
        let dist = dist as i64;
        for x in -dist..=dist {
            for y in -dist..=dist {
                let pos = BoardPosition {
                    x: self.x + x,
                    y: self.y + y,
                    z: self.z,
                };
                out.push(pos);
            }
        }
    }

    pub fn _xy_neighbors_buf_clamped(
        &self,
        dist: u32,
        out: &mut Vec<BoardPosition>,
        min_x: i64,
        max_x: i64,
        min_y: i64,
        max_y: i64,
    ) {
        out.clear();
        let dist = dist as i64;
        let x1 = (self.x - dist).clamp(min_x, max_x);
        let x2 = (self.x + dist).clamp(min_x, max_x);
        let y1 = (self.y - dist).clamp(min_y, max_y);
        let y2 = (self.y + dist).clamp(min_y, max_y);
        for x in x1..=x2 {
            for y in y1..=y2 {
                let pos = BoardPosition { x, y, z: self.z };
                out.push(pos);
            }
        }
    }

    pub fn is_valid(&self, map_size: (usize, usize, usize)) -> bool {
        self.x >= 0
            && self.x < map_size.0 as i64
            && self.y >= 0
            && self.y < map_size.1 as i64
            && self.z >= 0
            && self.z < map_size.2 as i64
    }

    pub fn iter_xy_neighbors_nosize(&self, dist: i64) -> NeighborsIterator {
        NeighborsIterator::new(self, dist, (0, 0), (2048, 2048))
    }

    pub fn iter_xy_neighbors(
        &self,
        dist: i64,
        map_size: (usize, usize, usize),
    ) -> NeighborsIterator {
        NeighborsIterator::new(
            self,
            dist,
            (0, 0),
            (
                map_size.0.saturating_sub(1) as i64,
                map_size.1.saturating_sub(1) as i64,
            ),
        )
    }
    pub fn iter_xy_neighbors_clamped(
        &self,
        dist: i64,
        from: (i64, i64),
        to: (i64, i64),
    ) -> NeighborsIterator {
        NeighborsIterator::new(self, dist, from, to)
    }

    pub fn _xy_neighbors_vec(&self, dist: u32) -> Vec<BoardPosition> {
        let mut ret: Vec<BoardPosition> = Vec::with_capacity((dist * dist * 4 + dist * 8) as usize);
        self._xy_neighbors_buf(dist, &mut ret);
        ret
    }

    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;
        let dz = self.z as f32 - other.z as f32;
        (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt()
    }

    pub fn distance2(&self, other: &Self) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;
        let dz = self.z as f32 - other.z as f32;
        dx.powi(2) + dy.powi(2) + dz.powi(2)
    }

    pub fn distance_taxicab(&self, other: &Self) -> i64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx.abs() + dy.abs() + dz.abs()
    }

    pub fn fast_distance_xy(&self, other: &Self) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        fastapprox::fast::pow(dx * dx + dy * dy, 0.5)
    }

    pub fn shadow_proximity(&self, shadow: &Self, tile: &Self) -> f32 {
        // This function assumes all points in the same Z plane.
        let sdx = self.x as f32 - shadow.x as f32;
        let sdy = self.y as f32 - shadow.y as f32;
        let sm = (sdx.powi(2) + sdy.powi(2)).sqrt();
        let tdx = self.x as f32 - tile.x as f32;
        let tdy = self.y as f32 - tile.y as f32;
        let tm = (tdx.powi(2) + tdy.powi(2)).sqrt();

        // Now convert tile vector into the same magnitude as the shadow vector:
        let tdx = tdx * sm / tm;
        let tdy = tdy * sm / tm;

        // The output of this function is the proximity scaled to the shadow point. Where
        // 0 .. 0.5 is full coverage, 1.0 is half coverage, and anything larger is no
        // coverage.
        let dx = tdx - sdx;
        let dy = tdy - sdy;
        (dx.powi(2) + dy.powi(2)).sqrt()
    }

    pub fn mini_hash(&self) -> f32 {
        let h: i64 = ((self.x + 41) % 61 + (self.y * 13 + 47) % 67 + (self.z * 29 + 59) % 79) % 109;
        h as f32 / 109.0
    }

    pub fn delta(&self, rhs: &BoardPosition) -> Direction {
        Direction {
            dx: (self.x - rhs.x) as f32,
            dy: (self.y - rhs.y) as f32,
            dz: (self.z - rhs.z) as f32,
        }
    }

    pub fn distance_to_chunk(&self, chunk: &(Range<usize>, Range<usize>, Range<usize>)) -> i64 {
        const Z_AXIS_MULTIPLIER: i64 = 8;

        let x = self.x;
        let y = self.y;
        let z = self.z;

        let x_range = chunk.0.start as i64..chunk.0.end as i64;
        let y_range = chunk.1.start as i64..chunk.1.end as i64;
        let z_range = chunk.2.start as i64..chunk.2.end as i64;

        let mut distance = 0;

        if !x_range.contains(&x) {
            distance += (x - x.clamp(x_range.start, x_range.end - 1)).abs();
        }

        if !y_range.contains(&y) {
            distance += (y - y.clamp(y_range.start, y_range.end - 1)).abs();
        }

        if !z_range.contains(&z) {
            distance += (z - z.clamp(z_range.start, z_range.end - 1)).abs() * Z_AXIS_MULTIPLIER;
        }

        distance
    }
}

/// A component that stores the board position of a map entity. This is used
/// to detect when the actual position of the sprite has moved from where it
/// was registered initially on the map_entity_field.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct MapEntityFieldBPos(pub BoardPosition);

#[derive(Debug, Clone)]
pub struct NeighborsIterator {
    current_x: i64,
    current_y: i64,
    min_x: i64,
    max_x: i64,
    max_y: i64,
    z: i64,
}

impl NeighborsIterator {
    pub fn new(
        center: &BoardPosition,
        max_distance: i64,
        from: (i64, i64),
        to: (i64, i64),
    ) -> NeighborsIterator {
        let min_x = (center.x - max_distance).max(from.0);
        let min_y = (center.y - max_distance).max(from.1);
        let max_x = (center.x + max_distance).min(to.0);
        let max_y = (center.y + max_distance).min(to.1);

        NeighborsIterator {
            current_x: min_x,
            current_y: min_y,
            min_x,
            max_x,
            max_y,
            z: center.z,
        }
    }
}

impl Iterator for NeighborsIterator {
    type Item = BoardPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y > self.max_y {
            return None;
        }

        let result = BoardPosition {
            x: self.current_x,
            y: self.current_y,
            z: self.z,
        };

        self.current_x += 1;
        if self.current_x > self.max_x {
            self.current_x = self.min_x;
            self.current_y += 1;
        }

        Some(result)
    }
}
