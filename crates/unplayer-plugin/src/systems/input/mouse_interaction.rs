use bevy::{
    input::mouse::MouseWheel,
    picking::events::{Out, Over, Pointer},
    prelude::*,
};
use uncore_board::behavior::component::Interactive;
use uncore_components::Toggleable;
use ungear::components::playergear::PlayerGear;
use unplayer_core::components::PlayerSprite;

pub(crate) fn mouse_right_click_gear_system(
    mouse: Res<ButtonInput<MouseButton>>,
    q_player: Query<&PlayerGear, With<PlayerSprite>>,
    mut q_toggleable: Query<&mut Toggleable>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        for player_gear in q_player.iter() {
            if let Some(entity) = player_gear.right_hand
                && let Ok(mut toggle) = q_toggleable.get_mut(entity)
            {
                toggle.is_on = !toggle.is_on;
            }
        }
    }
}

pub(crate) fn mouse_scroll_gear_system(
    mut scroll_events: MessageReader<MouseWheel>,
    mut q_player: Query<&mut PlayerGear, With<PlayerSprite>>,
) {
    for event in scroll_events.read() {
        if event.y != 0.0 {
            for mut player_gear in q_player.iter_mut() {
                if let Some(entity) = player_gear.right_hand.take() {
                    player_gear.inventory.push(entity);
                }
                if !player_gear.inventory.is_empty() {
                    player_gear.right_hand = Some(player_gear.inventory.remove(0));
                }
            }
        }
    }
}

pub(crate) fn mouse_over_interactive_system(
    mut events: MessageReader<Pointer<Over>>,
    mut q_interactive: Query<&mut Interactive>,
) {
    for event in events.read() {
        if let Ok(mut interactive) = q_interactive.get_mut(event.entity) {
            interactive.hovered = true;
        }
    }
}

pub(crate) fn mouse_out_interactive_system(
    mut events: MessageReader<Pointer<Out>>,
    mut q_interactive: Query<&mut Interactive>,
) {
    for event in events.read() {
        if let Ok(mut interactive) = q_interactive.get_mut(event.entity) {
            interactive.hovered = false;
        }
    }
}
