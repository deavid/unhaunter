use bevy::prelude::*;
use unprofile_core::profile::RuntimeInstallationId;
use unreplicon_core::resources::LocalPlayer;
pub(super) fn app_setup(app: &mut App) {
    // We must use PostStartup because the unprofile-plugin inserts RuntimeInstallationId in Startup.
    // However, if we are already in AppState::EngineBoot (which is the default),
    // OnEnter(AppState::EngineBoot) might have already fired before we reach PostStartup if
    // we were to put it there.
    //
    // The most robust way to ensure we catch the transition OR the initial state
    // is to run a one-shot initialization system that checks for the resource.
    app.add_systems(
        Update,
        set_local_player_system.run_if(
            resource_exists::<RuntimeInstallationId>
                .and(resource_exists_and_equals(LocalPlayer(None))),
        ),
    );
    app.add_systems(Update, observe_server_game_phase_transitions);
}

/// Initialise `LocalPlayer` from the `RuntimeInstallationId` inserted by the profile plugin.
fn set_local_player_system(
    runtime_id: Res<RuntimeInstallationId>,
    mut local_player: ResMut<LocalPlayer>,
) {
    *local_player = LocalPlayer(Some(runtime_id.0));
    info!("LocalPlayer identity set to UUID: {}", runtime_id.0);
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
