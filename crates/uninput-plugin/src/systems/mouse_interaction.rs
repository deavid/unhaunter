use bevy::{
    input::mouse::MouseWheel,
    picking::events::{Out, Over, Pointer},
    prelude::*,
};
use unbehavior::behavior::Interactive;
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSpectating, PlayerSprite};
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

pub fn mouse_over_interactive_system(
    mut events: MessageReader<Pointer<Over>>,
    mut q_interactive: Query<&mut Interactive>,
    q_spectator: Query<(), (With<MainPlayer>, With<PlayerSpectating>)>,
) {
    let is_spectator = !q_spectator.is_empty();
    for event in events.read() {
        if is_spectator {
            continue;
        }
        if let Ok(mut interactive) = q_interactive.get_mut(event.entity) {
            interactive.hovered = true;
        }
    }
}

pub fn mouse_out_interactive_system(
    mut events: MessageReader<Pointer<Out>>,
    mut q_interactive: Query<&mut Interactive>,
    q_spectator: Query<(), (With<MainPlayer>, With<PlayerSpectating>)>,
) {
    let is_spectator = !q_spectator.is_empty();
    for event in events.read() {
        if is_spectator {
            continue;
        }
        if let Ok(mut interactive) = q_interactive.get_mut(event.entity) {
            interactive.hovered = false;
        }
    }
}
