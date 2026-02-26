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
use unplayer_core::components::{PlayerInput, PlayerSprite};
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unreplicon_core::messages::{HostFloorGearDroppedEvent, HostFloorGearPickedUpEvent};
use unspatial_core::position::Position;

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
    mut ev_floor_pickup: MessageWriter<HostFloorGearPickedUpEvent>,
) {
    for (mut player_gear, player_pos, player_input) in players.iter_mut() {
        if player_input.grab {
            let mut closest = None;
            let mut min_dist = 1.0;

            for (entity, pos, gear_kind, behavior) in pickables.iter() {
                let dist = player_pos.distance(pos);
                if dist < min_dist {
                    min_dist = dist;
                    closest = Some((entity, pos, gear_kind, behavior));
                }
            }

            if let Some((entity, pos, gear_kind, behavior)) = closest {
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
                        commands.entity(entity).remove::<Sprite>();
                        commands.entity(entity).remove::<Transform>();
                        commands.entity(entity).remove::<Visibility>();
                        commands.entity(entity).remove::<GameSprite>();
                        commands.entity(entity).remove::<SpriteLayer>();
                        commands.entity(entity).remove::<MapColor>();
                        ev_sound.write(SoundEvent {
                            sound_file: "sounds/item-pickup-whoosh.ogg".to_string(),
                            volume: 1.0,
                            position: Some(*player_pos),
                            broadcast: true,
                        });
                        ev_floor_pickup.write(HostFloorGearPickedUpEvent {
                            pos: [pos.x, pos.y, pos.z],
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
                        broadcast: true,
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
    mut ev_floor_drop: MessageWriter<HostFloorGearDroppedEvent>,
    q_gear_kind: Query<&GearKind>,
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
                    broadcast: true,
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
                    broadcast: true,
                });
                if let Ok(kind) = q_gear_kind.get(entity) {
                    ev_floor_drop.write(HostFloorGearDroppedEvent {
                        kind: *kind,
                        pos: [player_pos.x, player_pos.y, player_pos.z],
                        direction: [
                            player_sprite.movement.dx,
                            player_sprite.movement.dy,
                            player_sprite.movement.dz,
                        ],
                    });
                }
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

pub(crate) fn app_setup(app: &mut App) {
    use unplayer_core::authoritative::PlayerAuthoritativeLogicSet;
    use untypes_core::states::AppState;
    app.add_systems(
        Update,
        (sync_held_gear_position, update_held_object_position).run_if(in_state(AppState::InGame)),
    );
    app.add_systems(
        Update,
        (grab_object, drop_object, cycle_inventory, swap_hands)
            .in_set(PlayerAuthoritativeLogicSet)
            .run_if(in_state(AppState::InGame)),
    );
}
