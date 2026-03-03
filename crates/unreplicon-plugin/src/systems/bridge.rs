use bevy::prelude::*;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::resources::LocalPlayer;
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::EngineBoot), set_local_player_system);
    app.add_systems(Update, observe_server_game_phase_transitions);
}

/// Initialise `LocalPlayer` based on the persistent profile.
fn set_local_player_system(profile: Option<Res<PlayerProfileData>>, mut commands: Commands) {
    if let Some(p) = profile {
        let uuid = p.installation_id;
        commands.insert_resource(LocalPlayer(Some(uuid)));
        info!("LocalPlayer identity set to UUID: {}", uuid);
    }
}

use unreplicon_core::components::ServerGamePhase;
use unreplicon_core::messages::RequestStartMission;

fn observe_server_game_phase_transitions(
    _q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    _local_player: Res<LocalPlayer>,
    _lobby: Query<&unreplicon_core::components::LobbyInfo>,
    _writer: MessageWriter<RequestStartMission>,
) {
    // This is a placeholder for logic that reacts to ServerGamePhase.
    // In a real scenario, this would bridge between server phase and client state.
}
