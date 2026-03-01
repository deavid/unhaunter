use bevy::prelude::*;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::resources::LocalPlayer;
use untypes_core::states::AppState;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::EngineBoot), set_local_player_system);
}

/// Initialise `LocalPlayer` based on the persistent profile.
fn set_local_player_system(profile: Option<Res<PlayerProfileData>>, mut commands: Commands) {
    if let Some(p) = profile {
        let uuid = p.installation_id;
        commands.insert_resource(LocalPlayer(Some(uuid)));
        info!("LocalPlayer identity set to UUID: {}", uuid);
    }
}

