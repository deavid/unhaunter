use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};
use unaudiospatial_core::emitter::LocalAudioEmitter;
use ungear_core::components::playergear::PlayerGear;
use uninput_core::components::PlayerInput;
use uninteraction_core::interaction::{Toggleable, Triggered};
use unplayer_core::components::{MainPlayer, PlayerSpectating, PlayerSprite};
use unreplicon_core::messages::ReplicatedSoundEvent;
use unreplicon_core::ownership::OwnerId;
use unreplicon_core::resources::AuthorityRole;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

pub(crate) fn toggle_gear_from_use_intent(
    mut commands: Commands,
    mut q_players: Query<
        (&PlayerGear, &mut PlayerInput, Option<&MainPlayer>, Has<InTruck>),
        (With<PlayerSprite>, Without<PlayerSpectating>),
    >,
    mut q_toggleable: Query<(&mut Toggleable, Option<&Position>)>,
    _ga: LocalAudioEmitter,
    mut ev_replicated_sound: MessageWriter<ToClients<ReplicatedSoundEvent>>,
    authority: Option<Res<AuthorityRole>>,
) {
    for (player_gear, mut player_input, main_player, in_truck) in q_players.iter_mut() {
        let is_main = main_player.is_some();
        if player_input.use_right_hand {
            player_input.use_right_hand = false;
            if let Some(entity) = player_gear.right_hand {
                debug!(
                    "player_gear_usage_system: Processing right-hand item {:?} (is_main={:?})",
                    entity, is_main
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

                        if is_main && authority.is_some() {
                            let sound_file = "sounds/switch-on-1.ogg".to_string();
                            let triggerer = OwnerId::Server; // Host player is server in this context

                            ev_replicated_sound.write(ToClients {
                                mode: SendMode::Broadcast,
                                message: ReplicatedSoundEvent {
                                    sound_file,
                                    volume: 1.0,
                                    position: pos.map(|p| [p.x, p.y, p.z]),
                                    triggerer,
                                    is_inside_truck: in_truck,
                                },
                            });
                        }
                    }
                }
                commands.entity(entity).insert(Triggered);
            }
        }
        if player_input.use_left_hand {
            player_input.use_left_hand = false;
            if let Some(entity) = player_gear.left_hand {
                debug!(
                    "player_gear_usage_system: Processing left-hand item {:?} (is_main={:?})",
                    entity, is_main
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

                        if is_main && authority.is_some() {
                            let sound_file = "sounds/switch-on-1.ogg".to_string();
                            let triggerer = OwnerId::Server;

                            ev_replicated_sound.write(ToClients {
                                mode: SendMode::Broadcast,
                                message: ReplicatedSoundEvent {
                                    sound_file,
                                    volume: 1.0,
                                    position: pos.map(|p| [p.x, p.y, p.z]),
                                    triggerer,
                                    is_inside_truck: in_truck,
                                },
                            });
                        }
                    }
                }
                commands.entity(entity).insert(Triggered);
            }
        }
    }
}
