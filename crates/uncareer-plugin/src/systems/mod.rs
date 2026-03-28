use bevy::prelude::*;
use uncareer_core::events::{CareerDeathRecordedEvent, CareerRewardCalculatedEvent};
use unprofile_core::events::DepositStakedEvent;
use untypes_core::roles::AuthorityRole;
use untypes_core::states::{AppState, SimulationState};

mod profile_update;
mod reward;

pub(crate) fn app_setup(app: &mut App) {
    // 1. Initialize resources
    app.init_resource::<reward::CurrentMissionDeposit>();

    // 2. Register events
    app.add_message::<CareerRewardCalculatedEvent>();
    app.add_message::<CareerDeathRecordedEvent>();
    app.add_message::<DepositStakedEvent>();

    // 3. Register systems
    app.add_systems(
        Update,
        (reward::store_deposit_stake, reward::record_career_death)
            .run_if(in_state(AppState::InGame)),
    );

    // Persist career outcomes to PlayerProfileData (no-op on server: profile is None)
    app.add_systems(
        Update,
        (
            profile_update::apply_career_reward_to_profile,
            profile_update::apply_career_death_to_profile,
        ),
    );

    // Authority (server) computes rewards during teardown
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        reward::calculate_and_emit_rewards.run_if(resource_exists::<AuthorityRole>),
    );

    // Clients (non-authority) compute rewards when entering summary screen
    // to populate their local SummaryData for display.
    app.add_systems(
        OnEnter(AppState::Summary),
        reward::calculate_and_emit_rewards.run_if(not(resource_exists::<AuthorityRole>)),
    );
}
