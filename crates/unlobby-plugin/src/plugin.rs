use bevy::prelude::*;
use unlobby_core::states::LobbyScreen;

pub struct UnhaunterLobbyPlugin;

impl Plugin for UnhaunterLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LobbyScreen>();
        crate::systems::setup::app_setup(app);
    }
}
