use bevy::prelude::*;
use ndarray::Array3;
use unboard_core::types::fielddata::LightFieldData;

#[derive(Resource, Debug, Clone)]
pub struct LightGrid {
    pub light_field: Array3<LightFieldData>,
    pub exposure_lux: f32,
    pub current_exposure: f32,
    pub current_exposure_accel: f32,
}

impl LightGrid {
    pub fn is_lit(&self, pos: unspatial_core::boardposition::BoardPosition) -> bool {
        let idx = pos.ndidx();
        if idx.0 >= self.light_field.shape()[0]
            || idx.1 >= self.light_field.shape()[1]
            || idx.2 >= self.light_field.shape()[2]
        {
            return false;
        }
        self.light_field[idx].lux > 0.1
    }
}

impl Default for LightGrid {
    fn default() -> Self {
        Self {
            light_field: Array3::from_elem((1, 1, 1), LightFieldData::default()),
            exposure_lux: 10.0,
            current_exposure: 10.0,
            current_exposure_accel: 0.0,
        }
    }
}
