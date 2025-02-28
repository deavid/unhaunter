use bevy::prelude::*;
use uncore::behavior::Behavior;
use uncore::components::board::position::Position;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::resources::board_data::BoardData;
use unlight::lighting::rebuild_lighting_field;
use unstd::plugins::board::rebuild_collision_data;

/// Updates the board field based on incoming events and rebuilds collision and lighting data if needed.
///
/// # Arguments
///
/// * `bf` - A mutable reference to the `BoardData` resource.
/// * `ev_bdr` - An event reader for `BoardDataToRebuild` events.
/// * `qt` - A query for entities with `Position` and `Behavior` components.
pub fn boardfield_update(
    mut bf: ResMut<BoardData>,
    mut ev_bdr: EventReader<BoardDataToRebuild>,
    qt: Query<(&Position, &Behavior)>,
) {
    if ev_bdr.is_empty() {
        return;
    }

    // Here we will recreate the field (if needed? - not sure how to detect that) ...
    // maybe add a timer since last update.
    let mut bdr = BoardDataToRebuild::default();

    // Merge all the incoming events into a single one.
    for b in ev_bdr.read() {
        if b.collision {
            bdr.collision = true;
        }
        if b.lighting {
            bdr.lighting = true;
        }
    }

    if bdr.collision {
        rebuild_collision_data(&mut bf, &qt);
    }

    if bdr.lighting {
        rebuild_lighting_field(&mut bf, &qt);
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(PostUpdate, boardfield_update)
        .add_event::<BoardDataToRebuild>();
}
