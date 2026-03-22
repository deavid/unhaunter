use bevy::prelude::*;
use ungear_core::resources::looking_gear::LookingGear;
use uninput_core::components::PlayerInputMapping;
use unplayer_core::components::MainPlayer;
use untypes_core::states::AppState;

fn system_update_looking_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut looking_gear: ResMut<LookingGear>,
    players: Query<&PlayerInputMapping, With<MainPlayer>>,
) {
    let Ok(input_mapping) = players.single() else {
        return;
    };
    if keyboard_input.just_pressed(input_mapping.controls.left_hand_toggle) {
        looking_gear.toggle();
    }

    looking_gear.held = keyboard_input.pressed(input_mapping.controls.left_hand_look);
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<LookingGear>().add_systems(
        Update,
        system_update_looking_gear.run_if(in_state(AppState::InGame)),
    );
}
