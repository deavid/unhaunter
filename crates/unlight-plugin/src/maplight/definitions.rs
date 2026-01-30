use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use ndarray::Array3;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;
use unfoundation_core::types::light::LightType;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

#[derive(Debug, Clone)]
pub(crate) struct FlashlightData {
    pub pos: Position,
    pub dir: Direction,
    pub power: f32,
    pub color: Color,
    pub light_type: LightType,
    pub vis_field: Array3<f32>,
}

#[derive(Resource, Default, Debug, Clone)]
pub(crate) struct ActiveFlashlights {
    pub list: Vec<FlashlightData>,
}

#[derive(SystemParam)]
pub(crate) struct GridResources<'w> {
    pub bf: Res<'w, BoardTopology>,
    pub bef: Res<'w, BoardEntityField>,
    pub bcf: Res<'w, BoardCollisionField>,
    pub miasma: Res<'w, MiasmaGrid>,
    pub miasma_config: Res<'w, MiasmaConfig>,
}
