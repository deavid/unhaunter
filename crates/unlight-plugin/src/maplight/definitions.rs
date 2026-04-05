use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};

#[derive(SystemParam)]
pub(crate) struct GridResources<'w> {
    pub bf: Res<'w, BoardTopology>,
    pub bcf: Res<'w, BoardCollisionField>,
}
