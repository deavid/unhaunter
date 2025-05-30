use bevy::prelude::*;
use uncore::states::AppState;
use uncore::types::gear_kind::GearKind;
use ungear::components::playergear::PlayerGear;
use unwalkiecore::{events::WalkieEvent, resources::WalkiePlay};

fn trigger_almost_ready_to_craft_repellent_system(
    player_query: Query<&PlayerGear>,
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
) {
    if let Ok(player_gear) = player_query.get_single() {
        for (gear, _epos) in player_gear.as_vec() {
            if gear.kind == GearKind::RepellentFlask {
                // If player already has a repellent flask, no need to prompt to craft.
                return;
            }
        }
    }

    walkie_play.set(
        WalkieEvent::JournalPointsToOneGhostNoCraft,
        time.elapsed_secs_f64(),
    );
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        trigger_almost_ready_to_craft_repellent_system.run_if(in_state(AppState::InGame)),
    );
}
