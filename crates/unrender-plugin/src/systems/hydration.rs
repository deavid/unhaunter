use bevy::prelude::*;
use unbehavior::behavior::{Behavior, Util};
use unbehavior::components;
use unboard_core::resources::roomdb::{RoomState, RoomStateMap, RoomTopology};
use unmetrics_core::metrics::SendMetric;
use untypes_core::hydration::HydrationStage;

use crate::metrics;

fn hydration_simulation_system(
    mut q: Query<(Entity, &Behavior, &unspatial_core::position::Position), With<HydrationStage<2>>>,
    mut roomtopo: ResMut<RoomTopology>,
    mut roomstate: ResMut<RoomStateMap>,
    mut commands: Commands,
) {
    let measure = metrics::HYDRATION_SIMULATION.time_measure();
    for (entity, behavior, pos) in q.iter_mut() {
        let mut cmd = commands.entity(entity);

        if behavior.p.is_floor {
            cmd.insert(components::Ground).insert(components::UVSurface);
        } else if behavior.p.is_wall || behavior.p.is_low_wall {
            cmd.insert(components::Collision)
                .insert(components::Opaque)
                .insert(components::UVSurface);
        }

        if let Util::RoomDef(name) = &behavior.p.util {
            roomtopo
                .room_tiles
                .insert(pos.to_board_position(), name.to_owned());
            roomstate.room_state.insert(name.clone(), RoomState::Off);
        }
    }
    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_simulation_system);
}
