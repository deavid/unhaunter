use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncareer_core::events::{CareerRewardCalculatedEvent, CareerDeathRecordedEvent};
use unmission_core::summary::SummaryData;
use unprofile_core::profile::PlayerProfileData;

pub(crate) fn apply_career_reward_to_profile(
    mut ev_reward: MessageReader<CareerRewardCalculatedEvent>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    mut sd: Option<ResMut<SummaryData>>,
) {
    for ev in ev_reward.read() {
        // 1. Update bank and deposit
        player_profile.progression.bank += ev.money_earned + ev.deposit_refunded;
        player_profile.progression.insurance_deposit = 0;

        // 2. Update XP and level
        player_profile.progression.player_xp += ev.xp_awarded;
        player_profile.progression.update_level();

        if ev.mission_successful {
            player_profile.statistics.total_missions_completed += 1;

            let map_stats = player_profile
                .map_statistics
                .entry(ev.map_path.clone())
                .or_default()
                .entry(ev.difficulty)
                .or_default();

            map_stats.total_missions_completed += 1;
            map_stats.total_mission_completed_time_seconds += ev.time_taken_secs as f64;
        }

        let map_stats = player_profile
            .map_statistics
            .entry(ev.map_path.clone())
            .or_default()
            .entry(ev.difficulty)
            .or_default();

        map_stats.total_play_time_seconds += ev.time_taken_secs as f64;
        map_stats.best_score = map_stats.best_score.max(ev.xp_awarded);
        map_stats.best_grade = map_stats.best_grade.max(ev.grade);

        player_profile.statistics.total_play_time_seconds += ev.time_taken_secs as f64;

        if let Some(ref mut sd) = sd {
            sd.final_bank_total = player_profile.progression.bank;
        }

        if let Err(e) = player_profile.persist() {
            warn!(
                "Failed to persist PlayerProfileData after career reward: {:?}",
                e
            );
        }
        info!("Profile: Applied career rewards for map {}", ev.map_path);
    }
}

pub(crate) fn apply_career_death_to_profile(
    mut ev_death: MessageReader<CareerDeathRecordedEvent>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    mut sd: Option<ResMut<SummaryData>>,
) {
    for ev in ev_death.read() {
        player_profile.progression.insurance_deposit = 0;
        player_profile.statistics.total_deaths += 1;

        let map_specific_stats = player_profile
            .map_statistics
            .entry(ev.map_path.clone())
            .or_default()
            .entry(ev.difficulty)
            .or_default();
        map_specific_stats.total_deaths += 1;

        if let Some(ref mut sd) = sd {
            sd.final_bank_total = player_profile.progression.bank;
        }

        if let Err(e) = player_profile.persist() {
            error!("Failed to persist PlayerProfileData after career death: {:?}", e);
        }
        info!("Profile: Recorded death for map {}", ev.map_path);
    }
}
