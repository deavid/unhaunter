use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::resources::board_topology::BoardTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unplayer_core::components::PlayerSprite;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::resources::LocalPlayer;
use unvitals_core::events::PlayerDiedEvent;

pub(crate) fn record_death_to_profile(
    mut ev_death: MessageReader<PlayerDiedEvent>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    local_player: Res<LocalPlayer>,
    q_players: Query<&PlayerSprite>,
    board_topology: Res<BoardTopology>,
    difficulty_res: Res<CurrentDifficulty>,
) {
    for ev in ev_death.read() {
        let player_uuid = q_players
            .iter()
            .find(|p| p.network_id == ev.id)
            .map(|p| p.id);

        if local_player.0 == player_uuid && player_uuid.is_some() {
            // It's us!
            player_profile.progression.insurance_deposit = 0;
            player_profile.statistics.total_deaths += 1;

            let map_path_str = board_topology.map_path.clone();
            let current_difficulty_variant = difficulty_res.0;

            let map_specific_stats = player_profile
                .map_statistics
                .entry(map_path_str.clone())
                .or_default()
                .entry(current_difficulty_variant)
                .or_default();
            map_specific_stats.total_deaths += 1;

            if let Err(e) = player_profile.persist() {
                error!("Failed to persist PlayerProfileData after death: {:?}", e);
            }
        }
    }
}
