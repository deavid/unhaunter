use bevy::prelude::*;
use uncareer_core::events::{CareerRewardCalculatedEvent, CareerDeathRecordedEvent, DepositStakedEvent};
use untypes_core::states::{AppState, SimulationState};
use untypes_core::roles::AuthorityRole;

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
        (
            reward::store_deposit_stake,
            reward::record_career_death,
        )
            .run_if(in_state(AppState::InGame)),
    );

    // Authority (server) computes rewards during teardown
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        reward::calculate_and_emit_rewards
            .run_if(resource_exists::<AuthorityRole>),
    );

    // Clients (non-authority) compute rewards when entering summary screen
    // to populate their local SummaryData for display.
    app.add_systems(
        OnEnter(AppState::Summary),
        reward::calculate_and_emit_rewards
            .run_if(not(resource_exists::<AuthorityRole>)),
    );
}
