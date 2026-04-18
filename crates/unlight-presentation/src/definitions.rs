use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;

#[derive(SystemParam)]
pub(crate) struct GridResources<'w> {
    pub bf: Res<'w, BoardTopology>,
    pub bef: Res<'w, BoardEntityField>,
    pub bcf: Res<'w, BoardCollisionField>,
    pub miasma: If<Res<'w, MiasmaGrid>>,
    pub miasma_config: Res<'w, MiasmaConfig>,
}
