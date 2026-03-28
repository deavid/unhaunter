use bevy::prelude::*;
use unmission_core::summary::SummaryData;
use untypes_core::states::AppState;

use crate::summary::{
    afk_timeout, cleanup, insert_afk_timer, keyboard, record_death_to_summary, remove_afk_timer,
    setup, setup_ui, store_mission_id, update_score, update_time, update_ui,
};

pub struct UnhaunterSummaryCorePlugin;

impl Plugin for UnhaunterSummaryCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SummaryData>()
            .add_systems(FixedUpdate, update_time.run_if(in_state(AppState::InGame)));
    }
}

pub struct UnhaunterSummaryPlugin;

impl Plugin for UnhaunterSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Summary),
            (setup, store_mission_id, setup_ui, insert_afk_timer).chain(),
        )
        .add_systems(OnExit(AppState::Summary), (cleanup, remove_afk_timer))
        .add_systems(
            Update,
            (
                keyboard,
                afk_timeout,
                update_ui,
                update_score,
                record_death_to_summary,
            )
                .run_if(in_state(AppState::Summary)),
        );
    }
}
