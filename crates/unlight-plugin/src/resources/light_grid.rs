use bevy::prelude::*;
use ndarray::{Array2, Array3};

use crate::types::light::LightFieldData;
use crate::types::prebaked_lighting_data::{PrebakedLightingData, PrebakedMetadata, WaveEdgeData};
use unbehavior::behavior::Behavior;

#[derive(Resource, Debug, Clone)]
pub struct LightGrid {
    pub light_field: Array3<LightFieldData>,
    pub exposure_lux: f32,
    pub current_exposure: f32,
    pub current_exposure_accel: f32,

    pub prebaked_lighting: Array3<PrebakedLightingData>,
    pub prebaked_metadata: PrebakedMetadata,
    pub prebaked_wave_edges: Vec<WaveEdgeData>,
    pub prebaked_propagation: Vec<Array2<[bool; 4]>>,
}

impl LightGrid {
    pub fn has_power(&self, qt: &Query<&Behavior>) -> bool {
        if self.prebaked_metadata.breakers.is_empty() {
            true
        } else {
            self.prebaked_metadata.breakers.iter().any(|&entity| {
                qt.get(entity)
                    .map(|behavior| behavior.state() == unbehavior::state::TileState::On)
                    .unwrap_or(false)
            })
        }
    }

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
            prebaked_lighting: Array3::from_elem((1, 1, 1), PrebakedLightingData::default()),
            prebaked_metadata: PrebakedMetadata::default(),
            prebaked_wave_edges: Vec::new(),
            prebaked_propagation: Vec::new(),
        }
    }
}
