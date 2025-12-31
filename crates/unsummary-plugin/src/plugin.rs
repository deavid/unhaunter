use bevy::prelude::*;
use unsummary_core::summary::SummaryData;
use untypes_core::states::AppState;

use crate::summary::{
    calculate_rewards_and_grades, cleanup, finalize_profile_update, keyboard, setup, setup_ui,
    store_mission_id, update_score, update_time, update_ui,
};

pub struct UnhaunterSummaryPlugin;

impl Plugin for UnhaunterSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SummaryData>()
            .add_systems(
                OnEnter(AppState::Summary),
                (
                    setup,
                    store_mission_id,
                    calculate_rewards_and_grades,
                    setup_ui,
                    finalize_profile_update,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppState::Summary), cleanup)
            .add_systems(FixedUpdate, update_time.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                (keyboard, update_ui, update_score).run_if(in_state(AppState::Summary)),
            );
    }
}
