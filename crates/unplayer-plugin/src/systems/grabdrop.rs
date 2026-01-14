use crate::components::player_sprite::PlayerSprite;
use bevy::prelude::*;
use unboard_core::behavior::Behavior;
use unboard_core::behavior::component::FloorItemCollidable;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_data::BoardData;
use unfoundation_core::types::gear::{EquipmentPosition, GearKind, Hand};
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::gear_stuff::GearStuff;
use ungear_core::resources::spawner::GearMarker;
use uninteraction_core::interaction::{Toggleable, Triggered};
use unplayer_core::components::HeldObject;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_type::SpriteType;
use unspatial_core::position::Position;

fn sync_held_gear_position(
    q_player: Query<(&Position, &PlayerGear), With<PlayerSprite>>,
    mut q_gear: Query<&mut Position, (With<GearMarker>, Without<PlayerSprite>)>,
) {
    for (player_pos, player_gear) in q_player.iter() {
        if let Some(e) = player_gear.left_hand
            && let Ok(mut gear_pos) = q_gear.get_mut(e)
        {
            *gear_pos = *player_pos;
        }
        if let Some(e) = player_gear.right_hand
            && let Ok(mut gear_pos) = q_gear.get_mut(e)
        {
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
        if let Some(held) = &player_gear.held_item
            && let Ok(mut object_pos) = q_held.get_mut(held.entity)
        {
            *object_pos = *player_pos;
            object_pos.z += 0.25;
        }
    }
}

fn grab_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite)>,
    pickables: Query<
        (Entity, &Position, Option<&GearKind>, Option<&Behavior>),
        (Without<PlayerSprite>, With<FloorItemCollidable>),
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (mut player_gear, player_pos, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.grab) {
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
                        commands.entity(entity).remove::<Sprite>();
                        commands.entity(entity).remove::<Transform>();
                        commands.entity(entity).remove::<Visibility>();
                        commands.entity(entity).remove::<GameSprite>();
                        commands.entity(entity).remove::<SpriteType>();
                        commands.entity(entity).remove::<MapColor>();
                        commands.spawn(AudioPlayer::new(
                            asset_server.load("sounds/item-pickup-whoosh.ogg"),
                        ));
                    }
                } else if let Some(behavior) = behavior
                    && behavior.p.object.pickable
                    && player_gear.held_item.is_none()
                {
                    player_gear.held_item = Some(HeldObject { entity });
                    commands.entity(entity).remove::<FloorItemCollidable>();
                    commands.spawn(AudioPlayer::new(
                        asset_server.load("sounds/item-pickup-whoosh.ogg"),
                    ));
                }
            }
        }
    }
}

fn drop_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite)>,
    mut commands: Commands,
    board_data: Res<BoardData>,
    pickables: Query<&Position, (With<FloorItemCollidable>, Without<PlayerSprite>)>,
    asset_server: Res<AssetServer>,
) {
    for (mut player_gear, player_pos, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.drop) {
            // Check if the tile is free
            let bpos = player_pos.to_board_position();
            let is_free = board_data
                .collision_field
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
                commands.spawn(AudioPlayer::new(
                    asset_server.load("sounds/item-drop-clunk.ogg"),
                ));
                continue;
            }

            if let Some(entity) = player_gear.right_hand.take() {
                commands.entity(entity).insert(*player_pos);
                commands.entity(entity).insert(FloorItemCollidable);
                commands.entity(entity).insert(EquipmentPosition::Deployed);
                commands.entity(entity).insert(DeployedGear {
                    direction: player_sprite.movement,
                });
                commands.spawn(AudioPlayer::new(
                    asset_server.load("sounds/item-drop-clunk.ogg"),
                ));
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

fn cycle_inventory(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &PlayerSprite)>,
    mut commands: Commands,
) {
    for (mut player_gear, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.cycle) {
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

fn swap_hands(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &PlayerSprite)>,
    mut commands: Commands,
) {
    for (mut player_gear, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.swap) {
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

fn item_trigger_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_player: Query<(&PlayerGear, &PlayerSprite)>,
    mut q_toggleable: Query<(&mut Toggleable, Option<&Position>)>,
    mut gs: GearStuff,
) {
    for (player_gear, player_sprite) in q_player.iter() {
        if keyboard_input.just_pressed(player_sprite.controls.right_hand_trigger)
            && let Some(entity) = player_gear.right_hand
        {
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                toggle.is_on = !toggle.is_on;
                if let Some(pos) = pos {
                    gs.play_audio("sounds/switch-on-1.ogg".into(), 1.0, pos);
                } else {
                    gs.play_audio_nopos("sounds/switch-on-1.ogg".into(), 1.0);
                }
            }
            commands.entity(entity).insert(Triggered);
        }
        if keyboard_input.just_pressed(player_sprite.controls.left_hand_trigger)
            && let Some(entity) = player_gear.left_hand
        {
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                toggle.is_on = !toggle.is_on;
                if let Some(pos) = pos {
                    gs.play_audio("sounds/switch-on-1.ogg".into(), 1.0, pos);
                } else {
                    gs.play_audio_nopos("sounds/switch-on-1.ogg".into(), 1.0);
                }
            }
            commands.entity(entity).insert(Triggered);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_held_gear_position,
            update_held_object_position,
            grab_object,
            drop_object,
            cycle_inventory,
            swap_hands,
            item_trigger_system,
        ),
    );
}
