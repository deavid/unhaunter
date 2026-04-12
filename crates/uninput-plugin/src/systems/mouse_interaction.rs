use bevy::{input::mouse::MouseWheel, prelude::*};
use uninput_core::components::PlayerInput;
use uninput_core::resources::MissionInputFocus;
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};

pub fn mouse_scroll_gear_system(
    mut scroll_events: MessageReader<MouseWheel>,
    mut q_player: Query<
        &mut PlayerInput,
        (
            With<PlayerSprite>,
            With<MainPlayer>,
            Without<PlayerSpectating>,
        ),
    >,
    focus: Res<MissionInputFocus>,
) {
    if !focus.has_focus {
        return;
    }
    for event in scroll_events.read() {
        if event.y < 0.0 {
            for mut player_input in q_player.iter_mut() {
                player_input.inventory_cycle = true;
            }
        } else if event.y > 0.0 {
            for mut player_input in q_player.iter_mut() {
                player_input.inventory_cycle_prev = true;
            }
        }
    }
}
