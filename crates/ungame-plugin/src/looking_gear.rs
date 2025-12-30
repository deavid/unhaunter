use bevy::prelude::*;
use uncore_resources::states::AppState;
use ungear_core::resources::looking_gear::LookingGear;
use unplayer_core::GameConfig;
use unplayer_core::components::PlayerSprite;

fn system_update_looking_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut looking_gear: ResMut<LookingGear>,
    gc: Res<GameConfig>,
    players: Query<&PlayerSprite>,
) {
    let Some(player_sprite) = players.iter().find(|player| player.id == gc.player_id) else {
        return;
    };
    if keyboard_input.just_pressed(player_sprite.controls.left_hand_toggle) {
        looking_gear.toggle();
    }

    looking_gear.held = keyboard_input.pressed(player_sprite.controls.left_hand_look);
}

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<LookingGear>().add_systems(
        Update,
        system_update_looking_gear.run_if(in_state(AppState::InGame)),
    );
}
