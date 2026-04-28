use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unmission_core::events::QuitMissionEvent;
use unmission_core::types::SimulationState;
use unreplicon_core::resources::LobbyPresenceRole;

pub(crate) fn handle_quit_mission(
    mut ev: MessageReader<QuitMissionEvent>,
    mut next_ui_state: ResMut<NextState<UIContextState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
) {
    for _ in ev.read() {
        next_sim_state.set(SimulationState::TearingDown);
        // Local-only exit path: this does not conclude the mission for other players.
        // lobby-presence means networked, offline goes to mission select.
        if lobby_presence.is_some() {
            next_ui_state.set(UIContextState::Lobby);
        } else {
            next_ui_state.set(UIContextState::MissionSelect);
        }
    }
}
