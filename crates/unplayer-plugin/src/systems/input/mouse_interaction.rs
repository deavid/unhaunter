use bevy::{
    input::mouse::MouseWheel,
    picking::events::{Out, Over, Pointer},
    prelude::*,
};
use unbehavior::components::Interactive;
use ungear_core::components::playergear::PlayerGear;
use uninteraction_core::interaction::{Toggleable, Triggered};
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSpectating, PlayerSprite};
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

pub(crate) fn player_gear_usage_system(
    mut commands: Commands,
    mut q_players: Query<
        (&PlayerGear, &mut PlayerInput, Option<&MainPlayer>),
        (With<PlayerSprite>, Without<PlayerSpectating>),
    >,
    mut q_toggleable: Query<(&mut Toggleable, Option<&Position>)>,
    mut ga: SoundEmitter,
    authority: Option<Res<untypes_core::roles::AuthorityRole>>,
) {
    let is_authority = authority.is_some();

    for (player_gear, player_input, main_player) in q_players.iter_mut() {
        let is_main = main_player.is_some();
        if player_input.use_right_hand
            && let Some(entity) = player_gear.right_hand
        {
            debug!(
                "player_gear_usage_system: Processing right-hand item {:?} (is_main={:?}, host={:?})",
                entity, is_main, is_authority
            );
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                let target_on = if is_main {
                    !toggle.is_on
                } else {
                    player_input
                        .target_right_hand
                        .as_ref()
                        .map(|(on, _)| *on)
                        .unwrap_or(toggle.is_on)
                };

                if toggle.is_on != target_on {
                    toggle.is_on = target_on;
                    if let Some(pos) = pos {
                        if is_main || is_authority {
                            // Host plays sound locally for remote player click
                            // and broadcasts it to other clients
                            ga.play_audio("sounds/switch-on-1.ogg".into(), 1.0, pos);
                        }
                    } else if is_main || is_authority {
                        ga.play_audio_nopos("sounds/switch-on-1.ogg".into(), 1.0);
                    }
                }
            }
            commands.entity(entity).insert(Triggered);
        }
        if player_input.use_left_hand
            && let Some(entity) = player_gear.left_hand
        {
            debug!(
                "player_gear_usage_system: Processing left-hand item {:?} (is_main={:?}, host={:?})",
                entity, is_main, is_authority
            );
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                let target_on = if is_main {
                    !toggle.is_on
                } else {
                    player_input
                        .target_left_hand
                        .as_ref()
                        .map(|(on, _)| *on)
                        .unwrap_or(toggle.is_on)
                };

                if toggle.is_on != target_on {
                    toggle.is_on = target_on;
                    if let Some(pos) = pos {
                        if is_main || is_authority {
                            // Host plays sound locally for remote player click
                            // and broadcasts it to other clients
                            ga.play_audio("sounds/switch-on-1.ogg".into(), 1.0, pos);
                        }
                    } else if is_main || is_authority {
                        ga.play_audio_nopos("sounds/switch-on-1.ogg".into(), 1.0);
                    }
                }
            }
            commands.entity(entity).insert(Triggered);
        }
    }
}

pub(crate) fn mouse_scroll_gear_system(
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

pub(crate) fn mouse_over_interactive_system(
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

pub(crate) fn mouse_out_interactive_system(
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
