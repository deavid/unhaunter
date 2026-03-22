use crate::exposure::ExposureModel;
use crate::types::light::LightFieldData;
use crate::types::prebaked_lighting_data::{PrebakedLightingData, PrebakedMetadata, WaveEdgeData};
use bevy::prelude::*;
use ndarray::Array3;
use unbehavior_core::behavior::Behavior;

#[derive(Resource, Debug, Clone)]
pub struct LightGrid {
    pub light_field: Array3<LightFieldData>,
    pub exposure: ExposureModel,

    pub prebaked_lighting: Array3<PrebakedLightingData>,
    pub prebaked_metadata: PrebakedMetadata,
    pub prebaked_wave_edges: Vec<WaveEdgeData>,
    pub prebaked_propagation: Vec<Array3<[bool; 4]>>,
}

impl LightGrid {
    pub fn reset(&mut self) {
        self.light_field = Array3::from_elem((1, 1, 1), LightFieldData::default());
        self.prebaked_lighting = Array3::from_elem((1, 1, 1), PrebakedLightingData::default());
        self.prebaked_metadata = PrebakedMetadata::default();
        self.prebaked_wave_edges.clear();
        self.prebaked_propagation.clear();
    }

    pub fn has_power(&self, qt: &Query<&Behavior>) -> bool {
        if self.prebaked_metadata.breakers.is_empty() {
            true
        } else {
            self.prebaked_metadata.breakers.iter().any(|&entity| {
                qt.get(entity)
                    .map(|behavior| behavior.state() == unbehavior_core::state::TileState::On)
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
            exposure: ExposureModel::default(),
            prebaked_lighting: Array3::from_elem((1, 1, 1), PrebakedLightingData::default()),
            prebaked_metadata: PrebakedMetadata::default(),
            prebaked_wave_edges: Vec::new(),
            prebaked_propagation: Vec::new(),
        }
    }
}
