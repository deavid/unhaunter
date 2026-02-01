use bevy::{
    input::mouse::MouseWheel,
    picking::events::{Out, Over, Pointer},
    prelude::*,
};
use unbehavior::components::Interactive;
use ungear_core::components::playergear::PlayerGear;
use uninteraction_core::interaction::{Authority, Toggleable, Triggered};
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unsound_core::emitter::SoundEmitter;
use unspatial_core::position::Position;
use untypes_core::cli::{CliOptions, is_host};

pub(crate) fn player_gear_usage_system(
    mut commands: Commands,
    q_players: Query<(&PlayerGear, &PlayerInput), With<PlayerSprite>>,
    mut q_toggleable: Query<(&mut Toggleable, Option<&Position>)>,
    mut ga: SoundEmitter,
    cli: Res<CliOptions>,
) {
    let authority = if is_host(cli) {
        Authority::Host
    } else {
        Authority::Client
    };

    for (player_gear, player_input) in q_players.iter() {
        if player_input.use_right_hand
            && let Some(entity) = player_gear.right_hand
        {
            debug!(
                "player_gear_usage_system: Toggling right-hand item {:?} on entity {:?} (authority={:?})",
                player_gear.right_hand, entity, authority
            );
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                toggle.is_on = !toggle.is_on;
                if authority == Authority::Host {
                    if let Some(pos) = pos {
                        ga.play_audio("sounds/switch-on-1.ogg".into(), 1.0, pos);
                    } else {
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
                "player_gear_usage_system: Toggling left-hand item {:?} on entity {:?} (authority={:?})",
                player_gear.left_hand, entity, authority
            );
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                toggle.is_on = !toggle.is_on;
                if authority == Authority::Host {
                    if let Some(pos) = pos {
                        ga.play_audio("sounds/switch-on-1.ogg".into(), 1.0, pos);
                    } else {
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
    mut q_player: Query<&mut PlayerInput, (With<PlayerSprite>, With<MainPlayer>)>,
) {
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
