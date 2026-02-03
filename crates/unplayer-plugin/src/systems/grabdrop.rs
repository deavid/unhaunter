use bevy::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::components::FloorItemCollidable;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::BoardCollisionField;
use unevents_core::events::sound::SoundEvent;
use unfoundation_core::types::gear::{EquipmentPosition, Hand};
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::{HeldObject, PlayerGear};
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::kind::GearKind;
use unnet_core::messages::{GrabRequestMsg, GrabResponseMsg, NetworkMessage, SendNetworkMessage};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::LocalPlayer;
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unspatial_core::position::Position;
use untypes_core::cli::{is_client, is_host};

fn sync_held_gear_position(
    q_player: Query<(&Position, &PlayerGear), With<PlayerSprite>>,
    mut q_gear: Query<&mut Position, (With<GearMarker>, Without<PlayerSprite>)>,
) {
    for (player_pos, player_gear) in q_player.iter() {
        if let Some(mut gear_pos) = player_gear.left_hand.and_then(|e| q_gear.get_mut(e).ok()) {
            *gear_pos = *player_pos;
        }
        if let Some(mut gear_pos) = player_gear.right_hand.and_then(|e| q_gear.get_mut(e).ok()) {
            *gear_pos = *player_pos;
        }
        for &e in &player_gear.inventory {
            if let Ok(mut gear_pos) = q_gear.get_mut(e) {
                *gear_pos = *player_pos;
            }
        }
    }
}

fn update_held_object_position(
    q_player: Query<(&Position, &PlayerGear), With<PlayerSprite>>,
    mut q_held: Query<&mut Position, (Without<PlayerSprite>, Without<GearMarker>)>,
) {
    for (player_pos, player_gear) in q_player.iter() {
        if let Some(mut object_pos) = player_gear
            .held_item
            .as_ref()
            .and_then(|held| q_held.get_mut(held.entity).ok())
        {
            *object_pos = *player_pos;
            object_pos.z += 0.25;
        }
    }
}

fn grab_object(
    mut players: Query<(&mut PlayerGear, &Position, &PlayerInput)>,
    pickables: Query<
        (Entity, &Position, Option<&GearKind>, Option<&Behavior>),
        (Without<PlayerSprite>, With<FloorItemCollidable>),
    >,
    mut commands: Commands,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    for (mut player_gear, player_pos, player_input) in players.iter_mut() {
        if player_input.grab {
            let mut closest = None;
            let mut min_dist = 1.0;

            for (entity, pos, gear_kind, behavior) in pickables.iter() {
                let dist = player_pos.distance(pos);
                if dist < min_dist {
                    min_dist = dist;
                    closest = Some((entity, gear_kind, behavior));
                }
            }

            if let Some((entity, gear_kind, behavior)) = closest {
                if gear_kind.is_some() {
                    let mut grabbed = false;
                    if player_gear.right_hand.is_none() {
                        player_gear.right_hand = Some(entity);
                        commands
                            .entity(entity)
                            .insert(EquipmentPosition::Hand(Hand::Right));
                        grabbed = true;
                    } else if player_gear.inventory.len() < 2 {
                        let old_item = player_gear.right_hand.replace(entity).unwrap();
                        player_gear.inventory.insert(0, old_item);
                        commands.entity(old_item).insert(EquipmentPosition::Stowed);
                        commands
                            .entity(entity)
                            .insert(EquipmentPosition::Hand(Hand::Right));
                        grabbed = true;
                    }

                    if grabbed {
                        commands.entity(entity).remove::<FloorItemCollidable>();
                        commands.entity(entity).remove::<DeployedGear>();
                        commands.entity(entity).remove::<Mesh2d>();
                        commands
                            .entity(entity)
                            .remove::<MeshMaterial2d<unrender_std::materials::CustomMaterial1>>();
                        commands.entity(entity).remove::<Transform>();
                        commands.entity(entity).remove::<Visibility>();
                        commands.entity(entity).remove::<GameSprite>();
                        commands.entity(entity).remove::<SpriteLayer>();
                        commands.entity(entity).remove::<MapColor>();
                        ev_sound.write(SoundEvent {
                            sound_file: "sounds/item-pickup-whoosh.ogg".to_string(),
                            volume: 1.0,
                            position: Some(*player_pos),
                        });
                    }
                } else if let Some(behavior) = behavior
                    && behavior.p.object.pickable
                    && player_gear.held_item.is_none()
                {
                    player_gear.held_item = Some(HeldObject { entity });
                    commands.entity(entity).remove::<FloorItemCollidable>();
                    ev_sound.write(SoundEvent {
                        sound_file: "sounds/item-pickup-whoosh.ogg".to_string(),
                        volume: 1.0,
                        position: Some(*player_pos),
                    });
                }
            }
        }
    }
}

fn drop_object(
    mut players: Query<(&mut PlayerGear, &Position, &PlayerInput, &PlayerSprite)>,
    mut commands: Commands,
    board_collision: Res<BoardCollisionField>,
    pickables: Query<&Position, (With<FloorItemCollidable>, Without<PlayerSprite>)>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    for (mut player_gear, player_pos, player_input, player_sprite) in players.iter_mut() {
        if player_input.drop {
            // Check if the tile is free
            let bpos = player_pos.to_board_position();
            let is_free = board_collision
                .0
                .get(bpos.ndidx())
                .map(|c| c.player_free)
                .unwrap_or(false);

            if !is_free {
                continue;
            }

            // Check for pile-ups
            let is_obstructed = pickables.iter().any(|pos| pos.distance(player_pos) < 0.5);
            if is_obstructed {
                continue;
            }

            if let Some(held) = player_gear.held_item.take() {
                let entity = held.entity;
                commands.entity(entity).insert(*player_pos);
                commands.entity(entity).insert(FloorItemCollidable);
                ev_sound.write(SoundEvent {
                    sound_file: "sounds/item-drop-clunk.ogg".to_string(),
                    volume: 1.0,
                    position: Some(*player_pos),
                });
                continue;
            }

            if let Some(entity) = player_gear.right_hand.take() {
                commands.entity(entity).insert(*player_pos);
                commands.entity(entity).insert(FloorItemCollidable);
                commands.entity(entity).insert(EquipmentPosition::Deployed);
                commands.entity(entity).insert(DeployedGear {
                    direction: player_sprite.movement,
                });
                ev_sound.write(SoundEvent {
                    sound_file: "sounds/item-drop-clunk.ogg".to_string(),
                    volume: 1.0,
                    position: Some(*player_pos),
                });
                if !player_gear.inventory.is_empty() {
                    let next_item = player_gear.inventory.remove(0);
                    player_gear.right_hand = Some(next_item);
                    commands
                        .entity(next_item)
                        .insert(EquipmentPosition::Hand(Hand::Right));
                }
            }
        }
    }
}

fn cycle_inventory(mut players: Query<(&mut PlayerGear, &PlayerInput)>, mut commands: Commands) {
    for (mut player_gear, player_input) in players.iter_mut() {
        if player_input.inventory_cycle {
            if let Some(entity) = player_gear.right_hand.take() {
                player_gear.inventory.push(entity);
                commands.entity(entity).insert(EquipmentPosition::Stowed);
            }
            if !player_gear.inventory.is_empty() {
                let entity = player_gear.inventory.remove(0);
                player_gear.right_hand = Some(entity);
                commands
                    .entity(entity)
                    .insert(EquipmentPosition::Hand(Hand::Right));
            }
        }
    }
}

fn swap_hands(mut players: Query<(&mut PlayerGear, &PlayerInput)>, mut commands: Commands) {
    for (mut player_gear, player_input) in players.iter_mut() {
        if player_input.inventory_swap {
            let tmp = player_gear.left_hand;
            player_gear.left_hand = player_gear.right_hand;
            player_gear.right_hand = tmp;
            if let Some(e) = player_gear.left_hand {
                commands
                    .entity(e)
                    .insert(EquipmentPosition::Hand(Hand::Left));
            }
            if let Some(e) = player_gear.right_hand {
                commands
                    .entity(e)
                    .insert(EquipmentPosition::Hand(Hand::Right));
            }
        }
    }
}

fn client_grab_object(
    players: Query<(&Position, &PlayerInput, &PlayerGear), With<MainPlayer>>,
    pickables: Query<
        (
            Entity,
            &Position,
            &NetworkId,
            Option<&GearKind>,
            Option<&Behavior>,
        ),
        (Without<PlayerSprite>, With<FloorItemCollidable>),
    >,
    local_id: Res<LocalPlayer>,
    mut ev_net: MessageWriter<SendNetworkMessage>,
) {
    let Some(player_id) = local_id.0 else {
        return;
    };
    for (player_pos, player_input, player_gear) in players.iter() {
        if player_input.grab {
            let mut closest = None;
            let mut min_dist = 1.0;

            for (entity, pos, net_id, gear_kind, behavior) in pickables.iter() {
                let dist = player_pos.distance(pos);
                if dist < min_dist {
                    min_dist = dist;
                    closest = Some((entity, net_id, gear_kind, behavior));
                }
            }

            if let Some((_entity, net_id, gear_kind, behavior)) = closest {
                let is_movable =
                    gear_kind.is_none() && behavior.map(|b| b.p.object.pickable).unwrap_or(false);

                if is_movable && player_gear.held_item.is_none() {
                    ev_net.write(SendNetworkMessage(NetworkMessage::GrabRequest(
                        GrabRequestMsg {
                            player_id,
                            target_id: *net_id,
                        },
                    )));
                }
            }
        }
    }
}

fn client_drop_object(
    players: Query<(&PlayerInput, &PlayerGear), With<MainPlayer>>,
    local_id: Res<LocalPlayer>,
    mut ev_net: MessageWriter<SendNetworkMessage>,
) {
    let Some(player_id) = local_id.0 else {
        return;
    };
    for (player_input, player_gear) in players.iter() {
        if player_input.drop && player_gear.held_item.is_some() {
            ev_net.write(SendNetworkMessage(NetworkMessage::DropRequest {
                player_id,
            }));
        }
    }
}

fn host_handle_grab_requests(
    mut commands: Commands,
    mut ev_reader: MessageReader<unnet_core::messages::NetworkDataEvent>,
    mut q_players: Query<(&Position, &mut PlayerGear, &NetworkId)>,
    q_pickables: Query<
        (Entity, &Position, &NetworkId, Option<&Behavior>),
        (With<FloorItemCollidable>, Without<PlayerSprite>),
    >,
    mut ev_net: MessageWriter<SendNetworkMessage>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::GrabRequest(msg) => {
                let GrabRequestMsg {
                    player_id,
                    target_id,
                } = msg;
                let mut p_data = None;
                for (pos, gear, id) in q_players.iter_mut() {
                    if id == player_id {
                        p_data = Some((pos, gear));
                        break;
                    }
                }
                let Some((player_pos, mut player_gear)) = p_data else {
                    continue;
                };

                let mut t_data = None;
                for (entity, pos, id, behavior) in q_pickables.iter() {
                    if id == target_id {
                        t_data = Some((entity, pos, behavior));
                        break;
                    }
                }

                let Some((target_entity, target_pos, _behavior)) = t_data else {
                    ev_net.write(SendNetworkMessage(NetworkMessage::GrabResponse(
                        GrabResponseMsg {
                            player_id: *player_id,
                            target_id: *target_id,
                            success: false,
                        },
                    )));
                    continue;
                };

                let dist = player_pos.distance(target_pos);
                let can_grab = dist < 1.0 && player_gear.held_item.is_none();

                if can_grab {
                    player_gear.held_item = Some(HeldObject {
                        entity: target_entity,
                    });
                    commands
                        .entity(target_entity)
                        .remove::<FloorItemCollidable>();
                    ev_sound.write(SoundEvent {
                        sound_file: "sounds/item-pickup-whoosh.ogg".to_string(),
                        volume: 1.0,
                        position: Some(*player_pos),
                    });
                    ev_net.write(SendNetworkMessage(NetworkMessage::GrabResponse(
                        GrabResponseMsg {
                            player_id: *player_id,
                            target_id: *target_id,
                            success: true,
                        },
                    )));
                } else {
                    ev_net.write(SendNetworkMessage(NetworkMessage::GrabResponse(
                        GrabResponseMsg {
                            player_id: *player_id,
                            target_id: *target_id,
                            success: false,
                        },
                    )));
                }
            }
            NetworkMessage::DropRequest { player_id } => {
                let mut p_data = None;
                for (pos, gear, id) in q_players.iter_mut() {
                    if id == player_id {
                        p_data = Some((pos, gear));
                        break;
                    }
                }
                let Some((player_pos, mut player_gear)) = p_data else {
                    continue;
                };

                if let Some(held) = player_gear.held_item.take() {
                    let entity = held.entity;
                    commands.entity(entity).insert(*player_pos);
                    commands.entity(entity).insert(FloorItemCollidable);
                    ev_sound.write(SoundEvent {
                        sound_file: "sounds/item-drop-clunk.ogg".to_string(),
                        volume: 1.0,
                        position: Some(*player_pos),
                    });
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (sync_held_gear_position, update_held_object_position),
    );
    app.add_systems(
        Update,
        (grab_object, drop_object, cycle_inventory, swap_hands).run_if(is_host),
    );
    app.add_systems(
        Update,
        (client_grab_object, client_drop_object).run_if(is_client),
    );
    app.add_systems(Update, host_handle_grab_requests.run_if(is_host));
}
