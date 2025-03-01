use crate::{
    celsius_to_kelvin,
    components::board::{boardposition::BoardPosition, position::Position},
    types::{
        board::{
            fielddata::{CollisionFieldData, LightFieldData},
            prebaked_lighting_data::PrebakedLightingData,
        },
        evidence::Evidence,
        miasma::MiasmaGrid,
    },
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use ndarray::Array3;

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
        }
    }
}
