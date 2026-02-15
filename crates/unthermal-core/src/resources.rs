use bevy::prelude::*;
use ndarray::Array3;

/// Configuration for the temperature diffusion system
#[derive(Debug, Clone)]
pub struct TemperatureDiffusionConfig {
    /// Minimum connectivity score (always processed)
    pub min_score: u8,
    /// Maximum connectivity score (rarely processed)
    pub max_score: u8,
    /// Default score for normal tiles
    pub default_score: u8,
    /// Score for stair tiles (critical for vertical flow)
    pub stair_score: u8,
    /// Score for closed doors (minimal processing)
    pub door_score: u8,
}

impl Default for TemperatureDiffusionConfig {
    fn default() -> Self {
        Self {
            min_score: 1,
            max_score: 32,
            default_score: 16,
            stair_score: 1,
            door_score: 32,
        }
    }
}

#[derive(Clone, Debug, Resource)]
pub struct ThermalGrid {
    pub temperature_field: Array3<f32>,
    /// Temperature activity/gradient magnitude per tile
    pub temperature_activity: Array3<f32>,
    /// Connectivity scores for temperature diffusion (one per tile)
    pub connectivity_scores: Array3<u8>,
    /// Configuration for temperature diffusion system
    pub temp_diffusion_config: TemperatureDiffusionConfig,
    pub ambient_temp: f32,
    /// List of indices of tiles that are valid for thermal simulation
    pub valid_tiles: Vec<(usize, usize, usize)>,
    /// Current index in the valid_tiles list for rolling updates
    pub iterator_index: usize,
}

impl ThermalGrid {
    pub fn reset(&mut self) {
        self.temperature_field = Array3::default((0, 0, 0));
        self.temperature_activity = Array3::default((0, 0, 0));
        self.connectivity_scores = Array3::default((0, 0, 0));
        self.valid_tiles.clear();
        self.iterator_index = 0;
    }
}

impl Default for ThermalGrid {
    fn default() -> Self {
        Self {
            temperature_field: Array3::default((0, 0, 0)),
            temperature_activity: Array3::default((0, 0, 0)),
            connectivity_scores: Array3::default((0, 0, 0)),
            temp_diffusion_config: TemperatureDiffusionConfig::default(),
            ambient_temp: 288.15, // 15°C in Kelvin
            valid_tiles: Vec::new(),
            iterator_index: 0,
        }
    }
}
