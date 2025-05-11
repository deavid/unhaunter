use crate::{
    celsius_to_kelvin,
    components::board::{boardposition::BoardPosition, position::Position},
    types::{
        board::{
            fielddata::{CollisionFieldData, LightFieldData},
            prebaked_lighting_data::{PrebakedLightingData, PrebakedMetadata, WaveEdgeData},
        },
        evidence::Evidence,
        miasma::MiasmaGrid,
    },
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use ndarray::{Array2, Array3};

#[derive(Clone, Debug, Resource)]
pub struct BoardData {
    pub map_size: (usize, usize, usize),
    pub origin: (i32, i32, i32),

    pub light_field: Array3<LightFieldData>,
    pub collision_field: Array3<CollisionFieldData>,
    pub temperature_field: Array3<f32>,
    pub sound_field: HashMap<BoardPosition, Vec<Vec2>>,
    pub map_entity_field: Array3<Vec<Entity>>,
    pub miasma: MiasmaGrid,
    pub breach_pos: Position,
    pub ambient_temp: f32,
    pub exposure_lux: f32,
    pub current_exposure: f32,
    pub current_exposure_accel: f32,
    pub evidences: HashSet<Evidence>,

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

    pub map_path: String, // Path to the current map file
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
        }
    }
}
