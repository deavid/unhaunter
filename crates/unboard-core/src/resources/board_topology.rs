use crate::types::fielddata::CollisionFieldData;
use crate::types::floor::FloorLevelMapping;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use ndarray::Array3;
use unspatial_core::boardposition::BoardPosition;

#[derive(Clone, Debug, Resource)]
pub struct BoardTopology {
    pub map_size: (usize, usize, usize),
    pub origin: (i32, i32, i32),

    pub ambient_temp: f32,

    // Floor mapping (Tiled floor number to z-index)
    pub floor_z_map: HashMap<i32, usize>, // Maps Tiled floor numbers to contiguous z indices
    pub z_floor_map: HashMap<usize, i32>, // Maps z indices back to Tiled floor numbers

    // Complete floor mapping information
    pub floor_mapping: FloorLevelMapping,

    pub map_path: String,      // Path to the current map file
    pub level_ready_time: f32, // Time when the level became ready
}

impl BoardTopology {
    pub fn reset(&mut self) {
        self.map_size = (0, 0, 0);
        self.origin = (0, 0, 0);
    }

    /// Check if a position is passable (for connectivity calculations)
    pub fn is_position_passable(&self, bcf: &BoardCollisionField, pos: BoardPosition) -> bool {
        if pos.x < 0
            || pos.y < 0
            || pos.z < 0
            || pos.x >= self.map_size.0 as i64
            || pos.y >= self.map_size.1 as i64
            || pos.z >= self.map_size.2 as i64
        {
            return false;
        }

        let collision_data = &bcf.0[pos.ndidx()];
        collision_data.player_free || collision_data.see_through
    }
}

#[derive(Clone, Debug, Resource, Default)]
pub struct BoardEntityField(pub Array3<Vec<Entity>>);

impl BoardEntityField {
    pub fn reset(&mut self) {
        self.0 = Array3::default((0, 0, 0));
    }
}

#[derive(Clone, Debug, Resource, Default)]
pub struct BoardCollisionField(pub Array3<CollisionFieldData>);

impl BoardCollisionField {
    pub fn reset(&mut self) {
        self.0 = Array3::default((0, 0, 0));
    }
}

impl FromWorld for BoardTopology {
    fn from_world(_world: &mut World) -> Self {
        // Using from_world to initialize is not needed but just in case we need it later.
        let map_size = (0, 0, 0);
        Self {
            map_size,
            origin: (0, 0, 0),
            ambient_temp: 288.15, // celsius_to_kelvin(15.0)
            floor_z_map: HashMap::new(),
            z_floor_map: HashMap::new(),
            floor_mapping: FloorLevelMapping {
                floor_to_z: HashMap::new(),
                z_to_floor: HashMap::new(),
                floor_display_names: HashMap::new(),
                ghost_attracting_objects: HashMap::new(),
                ghost_repelling_objects: HashMap::new(),
            },
            map_path: String::new(), // Initialize map_path with an empty string
            level_ready_time: 0.0,   // Initialize level_ready_time
        }
    }
}
