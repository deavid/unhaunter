use bevy::prelude::*;
use unboard_core::resources::board_topology::BoardTopology;
use uncareer_core::events::{CareerDeathRecordedEvent, CareerRewardCalculatedEvent};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unmission_core::summary::{ActiveMissionEvaluator, SummaryData};
use unplayer_core::components::PlayerSprite;
use unprofile_core::events::DepositStakedEvent;
use unreplicon_core::resources::LocalPlayer;
use unvitals_core::events::PlayerDiedEvent;

/// Local-only marker resource inserted when the local player dies during a mission.
/// Consumed by `calculate_and_emit_rewards` to force a failed payout,
/// preventing win-condition math from overwriting the death penalty.
/// Removed after consumption so it does not carry over to the next mission.
#[derive(Resource, Default, Debug)]
pub(crate) struct LocalPlayerDiedThisMission;

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
    evaluator: Option<Res<ActiveMissionEvaluator>>,
    deposit: Res<CurrentMissionDeposit>,
    mut ev_reward: MessageWriter<CareerRewardCalculatedEvent>,
    mut sd: ResMut<SummaryData>,
    died: Option<Res<LocalPlayerDiedThisMission>>,
    mut commands: Commands,
) {
    // If the local player died this mission, force a failed payout and skip
    // normal grade math. Remove the marker so it doesn't carry over.
    if died.is_some() {
        commands.remove_resource::<LocalPlayerDiedThisMission>();
        sd.mission_successful = false;
        sd.deposit_originally_held = deposit.amount;
        sd.deposit_returned_to_bank = 0;
        sd.costs_deducted_from_deposit = deposit.amount;
        sd.money_earned = 0;
        info!("Career: Local player died this mission — emitting failed reward");
        ev_reward.write(CareerRewardCalculatedEvent {
            money_earned: 0,
            xp_awarded: 0,
            deposit_refunded: 0,
            grade: sd.grade_achieved,
            map_path: sd.map_path.clone(),
            difficulty: sd.difficulty.0,
            mission_successful: false,
            time_taken_secs: sd.time_taken_secs,
        });
        return;
    }

    // Determine win condition locally: all ghost types exorcised.
    sd.mission_successful = sd.ghosts_unhaunted == sd.ghost_types.len() as u32;

    let Some(evaluator) = &evaluator else {
        warn!("Career: No ActiveMissionEvaluator found during reward calculation");
        return;
    };

    // The summary is local-only. Each player-bearing node computes its own
    // grade and reward fields when entering the summary screen.
    sd.calculate_score(evaluator.0.as_ref());
    let grade = evaluator.0.evaluate_grade(sd.full_score);

    // Note: mission_reward_base is currently populated locally from mission state.
    let grade_multiplier = grade.multiplier();
    let money_earned = if sd.mission_successful {
        (sd.mission_reward_base as f64 * grade_multiplier).round() as i64
    } else {
        0
    };

    // Deposit is only lost when the local player dies (KIA branch above returns early).
    // Leaving empty-handed is not punished by deposit loss.
    let deposit_refunded = deposit.amount;

    sd.grade_achieved = grade;
    sd.grade_multiplier = grade_multiplier;
    sd.money_earned = money_earned;
    sd.deposit_originally_held = deposit.amount;
    sd.deposit_returned_to_bank = deposit_refunded;
    sd.costs_deducted_from_deposit = deposit.amount - deposit_refunded;

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
    info!("Career: Emitted local reward event for grade {}", grade);
}

pub(crate) fn record_career_death(
    mut ev_death: MessageReader<PlayerDiedEvent>,
    local_player: Res<LocalPlayer>,
    q_players: Query<&PlayerSprite>,
    board_topology: Res<BoardTopology>,
    difficulty: Res<CurrentDifficulty>,
    mut ev_death_career: MessageWriter<CareerDeathRecordedEvent>,
    mut commands: Commands,
) {
    for ev in ev_death.read() {
        let player_uuid = q_players
            .iter()
            .find(|p| p.network_id == ev.id)
            .map(|p| p.id);

        if local_player.0 == player_uuid && player_uuid.is_some() {
            // Local player died! Mark for failed payout at summary time.
            commands.insert_resource(LocalPlayerDiedThisMission);

            // Emit career death event for persistence
            ev_death_career.write(CareerDeathRecordedEvent {
                map_path: board_topology.map_path.clone(),
                difficulty: difficulty.0,
            });

            info!("Career: Local player death recorded for economy");
        }
    }
}
