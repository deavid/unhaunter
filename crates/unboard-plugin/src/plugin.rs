use crate::systems;
use bevy::prelude::*;
use unboard_core::resources::roomdb::{RoomStateMap, RoomTopology};

pub struct UnhaunterBoardPlugin;

impl Plugin for UnhaunterBoardPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
        app.init_resource::<RoomTopology>()
            .init_resource::<RoomStateMap>();
    }
}
