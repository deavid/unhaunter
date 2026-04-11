use bevy::prelude::*;
use unmission_core::resources::MissionEndRequested;
use unmission_core::types::MissionEvent;
use unplayer_core::components::PlayerDisconnected;
use unplayer_core::components::PlayerInactive;
use unplayer_core::components::PlayerSpectating;
use unplayer_core::components::PlayerSprite;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

const EMPTY_MISSION_END_GRACE_SECS: f32 = 5.0;

pub(crate) fn evaluate_mission_end(
    query_players: Query<
        (
            &Position,
            Has<InTruck>,
            Has<PlayerSpectating>,
            Has<PlayerDisconnected>,
            Has<PlayerInactive>,
        ),
        With<PlayerSprite>,
    >,
    mut ev_mission: MessageWriter<MissionEvent>,
    mut mission_end_requested: ResMut<MissionEndRequested>,
    authority: Option<Res<unreplicon_core::resources::AuthorityRole>>,
    time: Res<Time>,
    mut empty_timer: Local<Option<f32>>,
) {
    let mut active_players = 0;
    let mut players_in_truck = 0;
    let mut disconnected_or_inactive_players = 0;

    for (_, in_truck, spectating, disconnected, inactive) in query_players.iter() {
        if disconnected || inactive {
            disconnected_or_inactive_players += 1;
            continue;
        }
        if spectating {
            continue;
        }
        active_players += 1;
        if in_truck {
            players_in_truck += 1;
        }
    }

    let all_in_truck = active_players > 0 && active_players == players_in_truck;

    // Condition A: Update availability for clients/UI
    mission_end_requested.0 = all_in_truck;

    // Condition B: all active players dead or gone
    if active_players == 0 {
        let now = time.elapsed_secs();
        let start = empty_timer.get_or_insert(now);
        if (now - *start) <= f32::EPSILON {
            debug!(
                "MISSION_END_EMPTY_TIMER_STARTED: active_players=0 truck_players={} disconnected_or_inactive={} authority={} time={:.3}",
                players_in_truck,
                disconnected_or_inactive_players,
                authority.is_some(),
                now
            );
        }
        if now - *start > EMPTY_MISSION_END_GRACE_SECS {
            if authority.is_some() {
                warn!(
                    "MISSION_END_EMPTY_TIMER_ELAPSED: requesting mission end after {:.3}s with active_players=0 disconnected_or_inactive={} truck_players={}",
                    now - *start,
                    disconnected_or_inactive_players,
                    players_in_truck
                );
                ev_mission.write(MissionEvent::End);
            }
            *empty_timer = None;
        }
    } else {
        if empty_timer.is_some() {
            debug!(
                "MISSION_END_EMPTY_TIMER_CLEARED: active_players={} truck_players={} disconnected_or_inactive={}",
                active_players, players_in_truck, disconnected_or_inactive_players
            );
        }
        *empty_timer = None;
    }
}
