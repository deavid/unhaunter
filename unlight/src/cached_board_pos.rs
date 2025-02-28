use std::f32::consts::PI;

use uncore::components::board::boardposition::BoardPosition;

/// A structure that pre-computes and caches geometric data for lighting and shadow calculations.
///
/// `CachedBoardPos` optimizes lighting performance by pre-computing distances, angles, and angle
/// ranges for all possible relative positions within a fixed-size window. This avoids expensive
/// trigonometric operations during real-time lighting calculations.
///
/// The cache uses a grid of size `SZ × SZ` (default 65×65) centered at position `CENTER` (default 32).
#[derive(Debug, Clone)]
pub struct CachedBoardPos {
    /// Cache of Euclidean distances from center to each position
    pub dist: [[f32; Self::SZ]; Self::SZ],

    /// Cache of discretized angles from center to each position
    pub angle: [[usize; Self::SZ]; Self::SZ],

    /// Cache of angle ranges (min, max) for shadow casting from each position
    pub angle_range: [[(i64, i64); Self::SZ]; Self::SZ],
}

impl CachedBoardPos {
    /// Center index of the cache grid
    const CENTER: i64 = 32;

    /// Size of the cache grid (CENTER * 2 + 1)
    const SZ: usize = (Self::CENTER * 2 + 1) as usize;

    /// The number of discrete angle steps around a circle
    ///
    /// This determines the angular resolution for shadow calculations.
    /// Higher values provide more precision but require more memory.
    pub const TAU_I: usize = 48 * 2;

    /// Creates a new `CachedBoardPos` with pre-computed values
    ///
    /// Initializes the cache by computing all distances and angles.
    /// This is computationally expensive but only needs to be done once.
    pub fn new() -> Self {
        let mut r = Self {
            dist: [[0.0; Self::SZ]; Self::SZ],
            angle: [[0; Self::SZ]; Self::SZ],
            angle_range: [[(0, 0); Self::SZ]; Self::SZ],
        };
        r.compute_angle();
        r.compute_dist();
        r
    }

    /// Computes the Euclidean distance from the center to each position
    ///
    /// Fills the `dist` array with distance values for every possible
    /// relative position in the grid.
    pub fn compute_dist(&mut self) {
        for (x, xv) in self.dist.iter_mut().enumerate() {
            for (y, yv) in xv.iter_mut().enumerate() {
                let x: f32 = x as f32 - Self::CENTER as f32;
                let y: f32 = y as f32 - Self::CENTER as f32;
                let dist: f32 = (x * x + y * y).sqrt();
                *yv = dist;
            }
        }
    }

    /// Computes angle and angle range values for each position
    ///
    /// Fills both the `angle` and `angle_range` arrays:
    /// - `angle`: The discrete angle index from center to the position
    /// - `angle_range`: The minimum and maximum angle deviations when considering
    ///   the cell's four corners, used for shadow casting
    pub fn compute_angle(&mut self) {
        for (x, xv) in self.angle.iter_mut().enumerate() {
            for (y, yv) in xv.iter_mut().enumerate() {
                let x: f32 = x as f32 - Self::CENTER as f32;
                let y: f32 = y as f32 - Self::CENTER as f32;
                let dist: f32 = (x * x + y * y).sqrt();
                let x = x / dist;
                let y = y / dist;
                let angle = x.acos() * y.signum() * Self::TAU_I as f32 / PI / 2.0;
                let angle_i = (angle.round() as i64).rem_euclid(Self::TAU_I as i64);
                *yv = angle_i as usize;
            }
        }
        for y in Self::CENTER - 3..=Self::CENTER + 3 {
            let mut v: Vec<usize> = vec![];
            for x in Self::CENTER - 3..=Self::CENTER + 3 {
                v.push(self.angle[x as usize][y as usize]);
            }
        }
        for (x, xv) in self.angle_range.iter_mut().enumerate() {
            for (y, yv) in xv.iter_mut().enumerate() {
                let orig_angle = self.angle[x][y];

                // if angle < Self::TAU_I / 4 || angle > Self::TAU_I - Self::TAU_I / 4 { // Angles
                // closer to zero need correction to avoid looking on the wrong place }
                let mut min_angle: i64 = 0;
                let mut max_angle: i64 = 0;
                let x: f32 = x as f32 - Self::CENTER as f32;
                let y: f32 = y as f32 - Self::CENTER as f32;
                for x1 in [x - 0.5, x + 0.5] {
                    for y1 in [y - 0.5, y + 0.5] {
                        let dist: f32 = (x1 * x1 + y1 * y1).sqrt();
                        let x1 = x1 / dist;
                        let y1 = y1 / dist;
                        let angle = x1.acos() * y1.signum() * Self::TAU_I as f32 / PI / 2.0;
                        let mut angle_i = angle.round() as i64 - orig_angle as i64;
                        if angle_i.abs() > Self::TAU_I as i64 / 2 {
                            angle_i -= Self::TAU_I as i64 * angle_i.signum();
                        }
                        min_angle = min_angle.min(angle_i);
                        max_angle = max_angle.max(angle_i);
                    }
                }
                *yv = (min_angle, max_angle);
            }
        }
        for y in Self::CENTER - 3..=Self::CENTER + 3 {
            let mut v: Vec<(i64, i64)> = vec![];
            for x in Self::CENTER - 3..=Self::CENTER + 3 {
                v.push(self.angle_range[x as usize][y as usize]);
            }
        }
    }

    /// Retrieves the cached distance between two board positions
    ///
    /// # Arguments
    ///
    /// * `s` - The source position
    /// * `d` - The destination position
    ///
    /// # Returns
    ///
    /// The pre-computed Euclidean distance between the positions
    pub fn bpos_dist(&self, s: &BoardPosition, d: &BoardPosition) -> f32 {
        let x = (d.x - s.x + Self::CENTER) as usize;
        let y = (d.y - s.y + Self::CENTER) as usize;

        // self.dist[x][y]
        unsafe { *self.dist.get_unchecked(x).get_unchecked(y) }
    }

    /// Retrieves the cached angle index between two board positions
    ///
    /// # Arguments
    ///
    /// * `s` - The source position
    /// * `d` - The destination position
    ///
    /// # Returns
    ///
    /// The pre-computed angle index (0 to TAU_I-1) representing the direction from source to destination
    pub fn bpos_angle(&self, s: &BoardPosition, d: &BoardPosition) -> usize {
        let x = (d.x - s.x + Self::CENTER) as usize;
        let y = (d.y - s.y + Self::CENTER) as usize;

        // self.angle[x][y]
        unsafe { *self.angle.get_unchecked(x).get_unchecked(y) }
    }

    /// Retrieves the cached angle range between two board positions
    ///
    /// The angle range represents the minimum and maximum angular deviations
    /// when considering the cell as a square rather than a point. This is used
    /// for accurate shadow casting.
    ///
    /// # Arguments
    ///
    /// * `s` - The source position
    /// * `d` - The destination position
    ///
    /// # Returns
    ///
    /// A tuple of (min_angle_offset, max_angle_offset) representing the angle range
    pub fn bpos_angle_range(&self, s: &BoardPosition, d: &BoardPosition) -> (i64, i64) {
        let x = (d.x - s.x + Self::CENTER) as usize;
        let y = (d.y - s.y + Self::CENTER) as usize;

        // self.angle_range[x][y]
        unsafe { *self.angle_range.get_unchecked(x).get_unchecked(y) }
    }
}

impl Default for CachedBoardPos {
    /// Creates a new instance with default values
    ///
    /// Equivalent to calling `CachedBoardPos::new()`
    fn default() -> Self {
        Self::new()
    }
}
