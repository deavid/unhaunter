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
fn boardfield_update(
    mut bf: ResMut<BoardData>,
    mut ev_bdr: EventReader<BoardDataToRebuild>,
    mut qt: Query<(Entity, &Position, &Behavior)>,
    mut avg_time: Local<(f32, f32)>,
) {
    if ev_bdr.is_empty() {
        return;
    }

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
        let mut lens = qt.transmute_lens::<(&Position, &Behavior)>();
        rebuild_lighting_field(&mut bf, &lens.query(), &mut avg_time);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(PostUpdate, boardfield_update)
        .add_event::<BoardDataToRebuild>();
}
