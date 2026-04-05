use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unmission_core::events::QuitMissionEvent;
use unreplicon_core::resources::LobbyPresenceRole;

pub(crate) fn handle_quit_mission(
    mut ev: MessageReader<QuitMissionEvent>,
    mut next_state: ResMut<NextState<UIContextState>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
) {
    for _ in ev.read() {
        // SP-6.2: navigate by role — lobby-presence means networked, offline goes to mission select.
        if lobby_presence.is_some() {
            next_state.set(UIContextState::Lobby);
        } else {
            next_state.set(UIContextState::MissionSelect);
        }
    }
}
