use bevy::prelude::*;
use unbehavior::behavior::{Behavior, Util};
use unbehavior::components;
use unbehavior::roomdb::RoomDB;
use unbehavior::state::TileState;
use untypes_core::hydration::HydrationStage;

fn hydration_simulation_system(
    mut q: Query<(Entity, &Behavior, &unspatial_core::position::Position), With<HydrationStage<2>>>,
    mut roomdb: ResMut<RoomDB>,
    mut commands: Commands,
) {
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
            roomdb
                .room_tiles
                .insert(pos.to_board_position(), name.to_owned());
            roomdb.room_state.insert(name.clone(), TileState::Off);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_simulation_system);
}
