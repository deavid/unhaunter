use bevy::prelude::*;
use unevents_core::events::mission::MissionEvent;
use unreplicon_core::resources::MissionEndRequested;
use unplayer_core::components::PlayerDisconnected;
use unplayer_core::components::PlayerInactive;
use unplayer_core::components::PlayerSpectating;
use unplayer_core::components::PlayerSprite;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

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
    time: Res<Time>,
    mut empty_timer: Local<Option<f32>>,
) {
    let mut active_players = 0;
    let mut players_in_truck = 0;

    for (_, in_truck, spectating, disconnected, inactive) in query_players.iter() {
        if disconnected || inactive {
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
        if now - *start > 2.0 {
            ev_mission.write(MissionEvent::End);
            *empty_timer = None;
        }
    } else {
        *empty_timer = None;
    }
}
