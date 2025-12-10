use crate::{
    celsius_to_kelvin,
    components::{
        board::{boardposition::BoardPosition, position::Position},
        ghost_behavior_dynamics::GhostBehaviorDynamics,
    },
    types::{
        board::{
            fielddata::{CollisionFieldData, LightFieldData},
            prebaked_lighting_data::{PrebakedLightingData, PrebakedMetadata, WaveEdgeData},
        },
        evidence::Evidence,
        miasma::MiasmaGrid,
    },
};
use bevy::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use ndarray::{Array2, Array3};

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
pub struct BoardData {
    pub map_size: (usize, usize, usize),
    pub origin: (i32, i32, i32),

    pub light_field: Array3<LightFieldData>,
    pub collision_field: Array3<CollisionFieldData>,
    pub temperature_field: Array3<f32>,
    /// Previous frame's temperature for gradient calculation
    pub temperature_field_prev: Array3<f32>,
    /// Temperature activity/gradient magnitude per tile
    pub temperature_activity: Array3<f32>,
    /// Connectivity scores for temperature diffusion (one per tile)
    pub connectivity_scores: Array3<u8>,
    /// Configuration for temperature diffusion system
    pub temp_diffusion_config: TemperatureDiffusionConfig,

    pub sound_field: HashMap<BoardPosition, Vec<Vec2>>,
    pub map_entity_field: Array3<Vec<Entity>>,
    pub miasma: MiasmaGrid,
    pub breach_pos: Position,
    pub ambient_temp: f32,
    pub exposure_lux: f32,
    pub current_exposure: f32,
    pub current_exposure_accel: f32,

    /// Evidences of the current ghost
    pub evidences: HashSet<Evidence>,
    pub ghost_dynamics: GhostBehaviorDynamics,

    // New prebaked lighting field.
    pub prebaked_lighting: Array3<PrebakedLightingData>,
    pub prebaked_metadata: PrebakedMetadata,
    pub prebaked_wave_edges: Vec<WaveEdgeData>,
    pub prebaked_propagation: Vec<Array2<[bool; 4]>>,

    // Floor mapping (Tiled floor number to z-index)
    pub floor_z_map: HashMap<i32, usize>, // Maps Tiled floor numbers to contiguous z indices
    pub z_floor_map: HashMap<usize, i32>, // Maps z indices back to Tiled floor numbers

    // Complete floor mapping information
    pub floor_mapping: crate::events::loadlevel::FloorLevelMapping,

    // Ghost warning state
    /// Current warning intensity (0.0-1.0)
    pub ghost_warning_intensity: f32,
    /// Source position of warning
    pub ghost_warning_position: Option<Position>,

    pub map_path: String,      // Path to the current map file
    pub level_ready_time: f32, // Time when the level became ready
}

impl BoardData {
    /// Returns if the given position has light above a fixed threshold.
    pub fn is_lit(&self, pos: BoardPosition) -> bool {
        if let Some(light_data) = self.light_field.get(pos.ndidx()) {
            light_data.lux > 0.5
        } else {
            false
        }
    }

    /// Calculate connectivity score for a tile at the given position
    /// Lower scores = higher processing frequency for temperature diffusion
    pub fn calculate_connectivity_score(
        &self,
        pos: BoardPosition,
        _roomdb: Option<&crate::resources::roomdb::RoomDB>,
    ) -> u8 {
        let config = &self.temp_diffusion_config;

        // Check bounds
        if pos.x < 0
            || pos.y < 0
            || pos.z < 0
            || pos.x >= self.map_size.0 as i64
            || pos.y >= self.map_size.1 as i64
            || pos.z >= self.map_size.2 as i64
        {
            return config.max_score; // Out of bounds tiles get minimal processing
        }

        let ndidx = pos.ndidx();
        let collision_data = &self.collision_field[ndidx];

        // Special case: stairs always get highest priority
        if collision_data.stair_offset != 0 {
            return config.stair_score;
        }

        // Special case: closed doors get minimal processing
        if collision_data.is_dynamic {
            return config.door_score;
        }

        if !collision_data.see_through {
            return config.max_score;
        }

        // Count passable 4-way neighbors
        let neighbors = [pos.left(), pos.right(), pos.top(), pos.bottom()];
        let mut passable_neighbors = 0u8;
        let mut second_degree_neighbors = 0u8;

        for neighbor in neighbors.iter() {
            if self.is_position_passable(neighbor.clone()) {
                passable_neighbors += 1;

                // Count second-degree neighbors (neighbors of neighbors)
                let second_neighbors = [
                    neighbor.left(),
                    neighbor.right(),
                    neighbor.top(),
                    neighbor.bottom(),
                ];
                for second_neighbor in second_neighbors.iter() {
                    if self.is_position_passable(second_neighbor.clone()) {
                        second_degree_neighbors += 1;
                    }
                }
            }
        }

        // Calculate total connectivity
        passable_neighbors + second_degree_neighbors
    }

    /// Check if a position is passable (for connectivity calculations)
    fn is_position_passable(&self, pos: BoardPosition) -> bool {
        if pos.x < 0
            || pos.y < 0
            || pos.z < 0
            || pos.x >= self.map_size.0 as i64
            || pos.y >= self.map_size.1 as i64
            || pos.z >= self.map_size.2 as i64
        {
            return false;
        }

        let collision_data = &self.collision_field[pos.ndidx()];
        collision_data.player_free || collision_data.see_through
    }

    /// Precompute connectivity scores for all tiles in the map
    pub fn precompute_connectivity_scores(
        &mut self,
        roomdb: Option<&crate::resources::roomdb::RoomDB>,
    ) {
        let mut score_distribution = HashMap::new();
        let mut total_tiles = 0;

        for x in 0..self.map_size.0 {
            for y in 0..self.map_size.1 {
                for z in 0..self.map_size.2 {
                    let pos = BoardPosition {
                        x: x as i64,
                        y: y as i64,
                        z: z as i64,
                    };
                    let score = self.calculate_connectivity_score(pos, roomdb);
                    self.connectivity_scores[(x, y, z)] = score;

                    // Track score distribution for logging
                    *score_distribution.entry(score).or_insert(0) += 1;
                    total_tiles += 1;
                }
            }
        }

        // Log score distribution for debugging
        info!("Connectivity score distribution for {} tiles:", total_tiles);
        for (score, count) in score_distribution.iter() {
            let percentage = (*count as f32 / total_tiles as f32) * 100.0;
            info!(
                "  Score {}: {} tiles ({:.1}% - {:.1}% processing chance)",
                score,
                count,
                percentage,
                100.0 / (*score as f32)
            );
        }
    }
}

impl FromWorld for BoardData {
    fn from_world(_world: &mut World) -> Self {
        // Using from_world to initialize is not needed but just in case we need it later.
        let map_size = (0, 0, 0);
        Self {
            map_size,
            origin: (0, 0, 0),
            collision_field: Array3::from_elem(map_size, CollisionFieldData::default()),
            light_field: Array3::from_elem(map_size, LightFieldData::default()),
            temperature_field: Array3::from_elem(map_size, 0.0),
            temperature_field_prev: Array3::from_elem(map_size, 0.0),
            temperature_activity: Array3::from_elem(map_size, 0.0),
            connectivity_scores: Array3::from_elem(map_size, 8), // Default to equivalent of current 1/8 selection
            temp_diffusion_config: TemperatureDiffusionConfig::default(),
            sound_field: HashMap::new(),
            exposure_lux: 1.0,
            current_exposure: 1.0,
            current_exposure_accel: 1.0,
            ambient_temp: celsius_to_kelvin(15.0),
            evidences: HashSet::new(),
            breach_pos: Position::new_i64(0, 0, 0),
            miasma: MiasmaGrid::default(),
            map_entity_field: Array3::default(map_size),
            prebaked_lighting: Array3::from_elem(map_size, PrebakedLightingData::default()),
            prebaked_metadata: PrebakedMetadata::default(),
            prebaked_wave_edges: Vec::new(),
            prebaked_propagation: Vec::new(),
            ghost_warning_intensity: 0.0,
            ghost_warning_position: None,
            floor_z_map: HashMap::new(),
            z_floor_map: HashMap::new(),
            floor_mapping: crate::events::loadlevel::FloorLevelMapping {
                floor_to_z: HashMap::new(),
                z_to_floor: HashMap::new(),
                floor_display_names: HashMap::new(),
                ghost_attracting_objects: HashMap::new(),
                ghost_repelling_objects: HashMap::new(),
            },
            map_path: String::new(), // Initialize map_path with an empty string
            level_ready_time: 0.0,   // Initialize level_ready_time
            ghost_dynamics: GhostBehaviorDynamics::default(),
        }
    }
}
