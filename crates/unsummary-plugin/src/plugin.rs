use bevy::prelude::*;
use unmission_core::summary::SummaryData;
use untypes_core::states::{AppState, SimulationState};

use crate::summary::{
    afk_timeout, calculate_rewards_and_grades, cleanup, finalize_profile_update, insert_afk_timer,
    keyboard, record_death_to_summary, remove_afk_timer, setup, setup_ui, store_mission_id,
    update_score, update_time, update_ui,
};

pub struct UnhaunterSummaryCorePlugin;

impl Plugin for UnhaunterSummaryCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SummaryData>()
            .add_systems(FixedUpdate, update_time.run_if(in_state(AppState::InGame)))
            .add_systems(
                OnEnter(SimulationState::TearingDown),
                calculate_rewards_and_grades
                    .run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
            );
    }
}

pub struct UnhaunterSummaryPlugin;

impl Plugin for UnhaunterSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Summary),
            (
                setup,
                store_mission_id,
                calculate_rewards_and_grades
                    .run_if(not(resource_exists::<untypes_core::roles::AuthorityRole>)),
                setup_ui,
                insert_afk_timer,
                finalize_profile_update,
            )
                .chain(),
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
