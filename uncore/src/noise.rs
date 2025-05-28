use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

/// Perlin noise resource with precomputed lookup table for fast access
#[derive(Resource, Debug)]
pub struct PerlinNoise {
    values: Vec<Vec<f32>>,
    resolution: f32,
    size: usize,
}

impl PerlinNoise {
    /// Create a new PerlinNoise with precomputed lookup table
    /// Uses a large table size and fine resolution to work for all use cases
    pub fn new(seed: u32) -> Self {
        const SIZE: usize = 4000;
        const RESOLUTION: f32 = 0.01;

        let perlin = Perlin::new(seed);
        let mut values = vec![vec![0.0; SIZE]; SIZE];

        for (x, row) in values.iter_mut().enumerate() {
            for (y, value) in row.iter_mut().enumerate() {
                *value =
                    perlin.get([x as f64 * RESOLUTION as f64, y as f64 * RESOLUTION as f64]) as f32;
            }
        }

        info!(
            "Precomputed Perlin noise lookup table initialized: {}x{} at resolution {} (~{} MB)",
            SIZE,
            SIZE,
            RESOLUTION,
            (SIZE * SIZE * 4) / 1_000_000
        );

        Self {
            values,
            resolution: RESOLUTION,
            size: SIZE,
        }
    }

    /// Get noise value from precomputed table
    pub fn get(&self, x: f32, y: f32) -> f32 {
        let xi = (x / self.resolution) as usize % self.size;
        let yi = (y / self.resolution) as usize % self.size;
        self.values[xi][yi]
    }
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self::new(1)
    }
}

// Noise frequency constants
pub const SHORT_TERM_NOISE_FREQ: f32 = 0.5;
pub const LONG_TERM_NOISE_FREQ: f32 = 0.07;
