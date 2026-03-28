use bevy::prelude::*;
use unaudiospatial_core::emitter::AudioEmitter;
use ungear_core::components::playergear::PlayerGear;
use uninput_core::components::PlayerInput;
use uninteraction_core::interaction::{Toggleable, Triggered};
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};
use unspatial_core::position::Position;

pub(crate) fn toggle_gear_from_use_intent(
    mut commands: Commands,
    mut q_players: Query<
        (&PlayerGear, &mut PlayerInput, Option<&MainPlayer>),
        (With<PlayerSprite>, Without<PlayerSpectating>),
    >,
    mut q_toggleable: Query<(&mut Toggleable, Option<&Position>)>,
    mut ga: AudioEmitter,
    authority: Option<Res<uncommon_app_core::roles::AuthorityRole>>,
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
                    // Remote players: no prediction data available, keep current state
                    toggle.is_on
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
                    // Remote players: no prediction data available, keep current state
                    toggle.is_on
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
