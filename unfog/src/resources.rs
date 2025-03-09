use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

#[derive(Resource, Debug, Clone)]
pub struct MiasmaConfig {
    pub initial_room_pressure: f32,
    pub initial_outside_pressure: f32,
    pub miasma_visibility_factor: f32,
    pub diffusion_rate: f32,
    pub velocity_scale: f32,
    pub inertia_factor: f32,
    pub friction: f32,
}

impl Default for MiasmaConfig {
    fn default() -> Self {
        Self {
            initial_room_pressure: 100.0,
            initial_outside_pressure: 0.0,
            miasma_visibility_factor: 0.14,
            diffusion_rate: 0.1,
            velocity_scale: 1000.0,
            inertia_factor: 1000.0,
            friction: -0.001,
        }
    }
}

// Define a new resource to store precomputed Perlin noise values
#[derive(Debug, Resource, Clone)]
pub struct PerlinNoiseTable {
    pub values: Vec<Vec<f32>>,
}

impl PerlinNoiseTable {
    const RES: f32 = 0.01;
    pub fn new(size: usize, seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        let mut values = vec![vec![0.0; size]; size];
        for (x, row) in values.iter_mut().enumerate() {
            for (y, value) in row.iter_mut().enumerate() {
                *value =
                    perlin.get([x as f64 * Self::RES as f64, y as f64 * Self::RES as f64]) as f32;
            }
        }
        PerlinNoiseTable { values }
    }

    pub fn get(&self, x: f32, y: f32) -> f32 {
        let xi = (x * Self::RES) as usize % self.values.len();
        let yi = (y * Self::RES) as usize % self.values[0].len();
        self.values[xi][yi]
    }
}

impl Default for PerlinNoiseTable {
    fn default() -> Self {
        Self::new(1000, 1)
    }
}
