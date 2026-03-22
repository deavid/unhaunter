use bevy::{input::mouse::MouseWheel, prelude::*};
use uninput_core::components::PlayerInput;
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};
use untruck_core::components::in_truck::InTruck;

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
    q_in_truck: Query<(), (With<MainPlayer>, With<InTruck>)>,
    game_state: Res<State<untypes_core::states::GameState>>,
) {
    if !q_in_truck.is_empty() || *game_state == untypes_core::states::GameState::Pause {
        return;
    }
    for event in scroll_events.read() {
        if event.y != 0.0 {
            for mut player_input in q_player.iter_mut() {
                player_input.inventory_cycle = true;
            }
        }
    }
}
