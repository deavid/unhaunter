use bevy::prelude::*;
use unboard_core::resources::board_topology::BoardTopology;
use uncareer_core::events::{CareerDeathRecordedEvent, CareerRewardCalculatedEvent};
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unmission_core::events::MissionCompletedEvent;
use unmission_core::summary::{ActiveMissionEvaluator, SummaryData};
use unplayer_core::components::PlayerSprite;
use unprofile_core::events::DepositStakedEvent;
use unreplicon_core::resources::AuthorityRole;
use unreplicon_core::resources::LocalPlayer;
use unvitals_core::events::PlayerDiedEvent;

/// A T2 runtime resource that caches the deposit amount for the current mission.
/// This bridges the T3b persistence state into T2 career logic.
#[derive(Resource, Default, Debug)]
pub(crate) struct CurrentMissionDeposit {
    pub amount: i64,
}

pub(crate) fn store_deposit_stake(
    mut ev: MessageReader<DepositStakedEvent>,
    mut local_deposit: ResMut<CurrentMissionDeposit>,
) {
    for ev in ev.read() {
        local_deposit.amount = ev.amount;
        debug!("Career: Stored deposit stake of ${}", ev.amount);
    }
}

pub(crate) fn calculate_and_emit_rewards(
    mut ev_completed: MessageReader<MissionCompletedEvent>,
    evaluator: Option<Res<ActiveMissionEvaluator>>,
    deposit: Res<CurrentMissionDeposit>,
    mut ev_reward: MessageWriter<CareerRewardCalculatedEvent>,
    mut sd: ResMut<SummaryData>,
    authority: Option<Res<AuthorityRole>>,
    app_state: Res<State<UIContextState>>,
) {
    // If we are on the summary screen and NOT authority, we still want to run this
    // to populate SummaryData for the local UI, even if the server already did it.
    // The event emission is what triggers persistence in unprofile-plugin.

    for _ in ev_completed.read() {
        let Some(evaluator) = &evaluator else {
            warn!("Career: No ActiveMissionEvaluator found during reward calculation");
            continue;
        };

        // 1. Compute score and grade
        // calculate_score must run here so full_score is populated before grading.
        // The authority never enters Summary state, so update_score() (which runs
        // only in Summary) cannot be relied upon to set full_score first.
        sd.calculate_score(evaluator.0.as_ref());
        let grade = evaluator.0.evaluate_grade(sd.full_score);

        // 2. Compute money earned
        // Note: mission_reward_base is currently set by unsummary-plugin's old logic
        // or by the map loader. We'll keep using it from SummaryData for now.
        let grade_multiplier = grade.multiplier();
        let money_earned = if sd.mission_successful {
            (sd.mission_reward_base as f64 * grade_multiplier).round() as i64
        } else {
            0
        };

        // 3. Compute deposit refund
        let deposit_refunded = if sd.mission_successful {
            deposit.amount
        } else {
            0
        };

        // 4. Update SummaryData for UI display
        sd.grade_achieved = grade;
        sd.grade_multiplier = grade_multiplier;
        sd.money_earned = money_earned;
        sd.deposit_originally_held = deposit.amount;
        sd.deposit_returned_to_bank = deposit_refunded;
        sd.costs_deducted_from_deposit = deposit.amount - deposit_refunded;

        // 5. Emit the career event (only if we are the authority or if it's a local game)
        // In a networked game, the server (authority) computes the "truth".
        if authority.is_some() || *app_state.get() == UIContextState::Summary {
            ev_reward.write(CareerRewardCalculatedEvent {
                money_earned,
                xp_awarded: sd.full_score,
                deposit_refunded,
                grade,
                map_path: sd.map_path.clone(),
                difficulty: sd.difficulty.0,
                mission_successful: sd.mission_successful,
                time_taken_secs: sd.time_taken_secs,
            });
            info!("Career: Emitted reward event for grade {}", grade);
        }
    }
}

pub(crate) fn record_career_death(
    mut ev_death: MessageReader<PlayerDiedEvent>,
    local_player: Res<LocalPlayer>,
    q_players: Query<&PlayerSprite>,
    board_topology: Res<BoardTopology>,
    difficulty: Res<CurrentDifficulty>,
    mut ev_death_career: MessageWriter<CareerDeathRecordedEvent>,
    mut sd: ResMut<SummaryData>,
) {
    for ev in ev_death.read() {
        let player_uuid = q_players
            .iter()
            .find(|p| p.network_id == ev.id)
            .map(|p| p.id);

        if local_player.0 == player_uuid && player_uuid.is_some() {
            // Local player died!

            // 1. Update SummaryData for UI
            sd.deposit_originally_held = 0; // Or keep it to show what was lost?
            // The current unsummary-plugin logic zeroes it out.
            sd.deposit_returned_to_bank = 0;
            sd.costs_deducted_from_deposit = 0;
            sd.money_earned = 0;

            // 2. Emit career death event for persistence
            ev_death_career.write(CareerDeathRecordedEvent {
                map_path: board_topology.map_path.clone(),
                difficulty: difficulty.0,
            });

            info!("Career: Local player death recorded for economy");
        }
    }
}
