use crate::components::player_sprite::PlayerSprite;
use bevy::prelude::*;
use uncore_board::behavior::component::FloorItemCollidable;
use uncore_components::Triggered;
use uncore_foundation::types::gear::{EquipmentPosition, GearKind, Hand};
use ungear::components::deployedgear::DeployedGear;
use ungear::components::playergear::PlayerGear;
use ungear::resources::spawner::GearMarker;
use unspatial::Position;

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

fn grab_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite)>,
    pickables: Query<
        (Entity, &Position, &GearKind),
        (Without<PlayerSprite>, With<FloorItemCollidable>),
    >,
    mut commands: Commands,
) {
    for (mut player_gear, player_pos, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.grab) {
            let mut closest = None;
            let mut min_dist = 1.0;

            for (entity, pos, _kind) in pickables.iter() {
                let dist = player_pos.distance(pos);
                if dist < min_dist {
                    min_dist = dist;
                    closest = Some(entity);
                }
            }

            if let Some(entity) = closest {
                if player_gear.right_hand.is_none() {
                    player_gear.right_hand = Some(entity);
                    commands.entity(entity).remove::<FloorItemCollidable>();
                    commands.entity(entity).remove::<DeployedGear>();
                    commands
                        .entity(entity)
                        .insert(EquipmentPosition::Hand(Hand::Right));
                } else if player_gear.left_hand.is_none() {
                    player_gear.left_hand = Some(entity);
                    commands.entity(entity).remove::<FloorItemCollidable>();
                    commands.entity(entity).remove::<DeployedGear>();
                    commands
                        .entity(entity)
                        .insert(EquipmentPosition::Hand(Hand::Left));
                }
            }
        }
    }
}

fn drop_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite)>,
    mut commands: Commands,
) {
    for (mut player_gear, player_pos, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.drop) {
            if let Some(entity) = player_gear.right_hand.take() {
                commands.entity(entity).insert(*player_pos);
                commands.entity(entity).insert(FloorItemCollidable);
                commands.entity(entity).insert(EquipmentPosition::Deployed);
            } else if let Some(entity) = player_gear.left_hand.take() {
                commands.entity(entity).insert(*player_pos);
                commands.entity(entity).insert(FloorItemCollidable);
                commands.entity(entity).insert(EquipmentPosition::Deployed);
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
) {
    for (player_gear, player_sprite) in q_player.iter() {
        if keyboard_input.just_pressed(player_sprite.controls.right_hand_trigger)
            && let Some(entity) = player_gear.right_hand
        {
            commands.entity(entity).insert(Triggered);
        }
        if keyboard_input.just_pressed(player_sprite.controls.left_hand_trigger)
            && let Some(entity) = player_gear.left_hand
        {
            commands.entity(entity).insert(Triggered);
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_held_gear_position,
            grab_object,
            drop_object,
            cycle_inventory,
            swap_hands,
            item_trigger_system,
        ),
    );
}
