use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore_resources::states::AppState;
use ungear::components::playergear::PlayerGear;
use ungear::resources::looking_gear::LookingGear;
use unplayer_core::components::PlayerSprite;
use unprofile_plugin::data::PlayerProfileData;

fn acknowledge_blinking_gear_hint_system(
    _keyboard_input: Res<ButtonInput<KeyCode>>,
    _player_query: Query<(&PlayerSprite, &PlayerGear)>,
    _profile_data: ResMut<Persistent<PlayerProfileData>>,
    _looking_gear: Res<LookingGear>,
) {
    // STUB: Needs to be re-implemented with ECS entities
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        acknowledge_blinking_gear_hint_system.run_if(in_state(AppState::InGame)),
    );
}
