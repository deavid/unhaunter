use bevy::prelude::*;
use uncommon_app_core::roles::LobbyPresenceRole;
use uncommon_app_core::states::AppState;
use unmission_core::events::QuitMissionEvent;

pub(crate) fn handle_quit_mission(
    mut ev: MessageReader<QuitMissionEvent>,
    mut next_state: ResMut<NextState<AppState>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
) {
    for _ in ev.read() {
        // SP-6.2: navigate by role — lobby-presence means networked, offline goes to mission select.
        if lobby_presence.is_some() {
            next_state.set(AppState::Lobby);
        } else {
            next_state.set(AppState::MissionSelect);
        }
    }
}
