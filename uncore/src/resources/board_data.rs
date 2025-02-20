use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    components::board::{boardposition::BoardPosition, position::Position},
    types::{
        board::fielddata::{CollisionFieldData, LightFieldData},
        evidence::Evidence,
        miasma::MiasmaGrid,
    },
};

#[derive(Clone, Debug, Resource)]
pub struct BoardData {
    pub map_size: (usize, usize),
    pub origin: (i32, i32),

    pub light_field: HashMap<BoardPosition, LightFieldData>,
    pub collision_field: HashMap<BoardPosition, CollisionFieldData>,
    pub temperature_field: HashMap<BoardPosition, f32>,
    pub sound_field: HashMap<BoardPosition, Vec<Vec2>>,
    pub miasma: MiasmaGrid,
    pub breach_pos: Position,
    pub ambient_temp: f32,
    pub exposure_lux: f32,
    pub current_exposure: f32,
    pub current_exposure_accel: f32,
    pub evidences: HashSet<Evidence>,
}

impl FromWorld for BoardData {
    fn from_world(_world: &mut World) -> Self {
        // Using from_world to initialize is not needed but just in case we need it later.
        Self {
            map_size: (0, 0),
            origin: (0, 0),
            collision_field: HashMap::new(),
            light_field: HashMap::new(),
            temperature_field: HashMap::new(),
            sound_field: HashMap::new(),
            exposure_lux: 1.0,
            current_exposure: 1.0,
            current_exposure_accel: 1.0,
            ambient_temp: 15.0,
            evidences: HashSet::new(),
            breach_pos: Position::new_i64(0, 0, 0),
            miasma: MiasmaGrid::default(),
        }
    }
}
