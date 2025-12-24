use crate::components::player_sprite::PlayerSprite;
use bevy::prelude::*;
use uncore_board::behavior::component::FloorItemCollidable;
use uncore_components::{Flashlight, Toggleable, Triggered};
use uncore_foundation::types::gear::GearKind;
use ungear::components::deployedgear::DeployedGear;
use ungear::components::playergear::PlayerGear;
use unspatial::Position;

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
                    commands.entity(entity).remove::<Position>();
                    commands.entity(entity).remove::<FloorItemCollidable>();
                    commands.entity(entity).remove::<DeployedGear>();
                } else if player_gear.left_hand.is_none() {
                    player_gear.left_hand = Some(entity);
                    commands.entity(entity).remove::<Position>();
                    commands.entity(entity).remove::<FloorItemCollidable>();
                    commands.entity(entity).remove::<DeployedGear>();
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
            } else if let Some(entity) = player_gear.left_hand.take() {
                commands.entity(entity).insert(*player_pos);
                commands.entity(entity).insert(FloorItemCollidable);
            }
        }
    }
}

fn cycle_inventory(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &PlayerSprite)>,
) {
    for (mut player_gear, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.cycle) {
            if let Some(entity) = player_gear.right_hand.take() {
                player_gear.inventory.push(entity);
            }
            if !player_gear.inventory.is_empty() {
                player_gear.right_hand = Some(player_gear.inventory.remove(0));
            }
        }
    }
}

fn swap_hands(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &PlayerSprite)>,
) {
    for (mut player_gear, player_sprite) in players.iter_mut() {
        if keyboard_input.just_pressed(player_sprite.controls.swap) {
            let tmp = player_gear.left_hand;
            player_gear.left_hand = player_gear.right_hand;
            player_gear.right_hand = tmp;
        }
    }
}

fn item_trigger_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_player: Query<(&PlayerGear, &PlayerSprite)>,
) {
    for (player_gear, player_sprite) in q_player.iter() {
        if keyboard_input.just_pressed(player_sprite.controls.trigger)
            && let Some(entity) = player_gear.right_hand
        {
            commands.entity(entity).insert(Triggered);
        }
    }
}

fn torch_toggle_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_toggleable: Query<&mut Toggleable>,
    q_player: Query<(&PlayerGear, &PlayerSprite)>,
    q_flashlight: Query<Entity, With<Flashlight>>,
) {
    for (player_gear, player_sprite) in q_player.iter() {
        if keyboard_input.just_pressed(player_sprite.controls.torch) {
            let mut flashlight_entity = None;
            if let Some(e) = player_gear.right_hand
                && q_flashlight.contains(e)
            {
                flashlight_entity = Some(e);
            }
            if flashlight_entity.is_none()
                && let Some(e) = player_gear.left_hand
                && q_flashlight.contains(e)
            {
                flashlight_entity = Some(e);
            }
            if flashlight_entity.is_none() {
                for &e in &player_gear.inventory {
                    if q_flashlight.contains(e) {
                        flashlight_entity = Some(e);
                        break;
                    }
                }
            }

            if let Some(entity) = flashlight_entity
                && let Ok(mut toggle) = q_toggleable.get_mut(entity)
            {
                toggle.is_on = !toggle.is_on;
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            grab_object,
            drop_object,
            cycle_inventory,
            swap_hands,
            item_trigger_system,
            torch_toggle_system,
        ),
    );
}
