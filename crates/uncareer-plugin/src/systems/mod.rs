use bevy::prelude::*;
use uncareer_core::events::{CareerDeathRecordedEvent, CareerRewardCalculatedEvent};
use uncommon_states_core::UIContextState;
use unprofile_core::events::DepositStakedEvent;
use unreplicon_core::resources::LocalPlayerRole;

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
            .run_if(in_state(UIContextState::InGame)),
    );

    // Persist career outcomes to PlayerProfileData (no-op on server: profile is None)
    app.add_systems(
        Update,
        (
            profile_update::apply_career_reward_to_profile,
            profile_update::apply_career_death_to_profile,
        ),
    );

    // Summary is local-only and is built on player-bearing nodes when the
    // summary screen is entered.
    app.add_systems(
        OnEnter(UIContextState::Summary),
        reward::calculate_and_emit_rewards.run_if(resource_exists::<LocalPlayerRole>),
    );
}
