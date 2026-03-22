use bevy::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use unbehavior::behavior::Behavior;
use unbehavior::components::FloorItemCollidable;
use unboard_core::resources::board_topology::BoardCollisionField;
use ungear_core::components::playergear::{HeldObject, PlayerGear};
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::kind::GearKind;
use ungear_core::types::gear::{EquipmentPosition, Hand};
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unreplicon_core::messages::{RequestDrop, RequestGrab};
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::position::Position;

fn sync_inventory_position_to_holder(
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

fn sync_held_object_position_to_holder(
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

fn queue_pickup_request(
    players: Query<(&PlayerGear, &Position, &PlayerInput)>,
    pickables: Query<
        (Entity, &Position, Option<&GearKind>, Option<&Behavior>),
        (Without<PlayerSprite>, With<FloorItemCollidable>),
    >,
    mut writer_grab: MessageWriter<RequestGrab>,
) {
    for (player_gear, player_pos, player_input) in players.iter() {
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

            if let Some((entity, _pos, gear_kind, behavior)) = closest {
                if gear_kind.is_some() {
                    let can_grab_gear =
                        player_gear.right_hand.is_none() || player_gear.inventory.len() < 2;
                    if can_grab_gear {
                        writer_grab.write(RequestGrab { entity });
                    }
                } else if let Some(behavior) = behavior
                    && behavior.p.object.pickable
                    && player_gear.held_item.is_none()
                {
                    writer_grab.write(RequestGrab { entity });
                }
            }
        }
    }
}

fn queue_drop_request(
    mut players: Query<(&mut PlayerGear, &Position, &PlayerInput, &PlayerSprite)>,
    mut commands: Commands,
    board_collision: Res<BoardCollisionField>,
    pickables: Query<&Position, (With<FloorItemCollidable>, Without<PlayerSprite>)>,
    mut writer_drop: MessageWriter<RequestDrop>,
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

            let mut dropped_gear = false;
            let entity = if let Some(held) = player_gear.held_item.take() {
                held.entity
            } else if let Some(right_hand) = player_gear.right_hand.take() {
                dropped_gear = true;
                right_hand
            } else {
                continue;
            };

            commands.entity(entity).remove::<LocallyOwned>();
            writer_drop.write(RequestDrop {
                entity,
                position: [player_pos.x, player_pos.y, player_pos.z],
                direction: [
                    player_sprite.movement.dx,
                    player_sprite.movement.dy,
                    player_sprite.movement.dz,
                ],
            });
            ev_sound.write(SoundEvent {
                sound_file: "sounds/item-drop-clunk.ogg".to_string(),
                volume: 1.0,
                position: Some(*player_pos),
                broadcast: false, // Visual spawn broadcast handles remote clients; this is just local feedback
            });

            if dropped_gear && !player_gear.inventory.is_empty() {
                let next_item = player_gear.inventory.remove(0);
                player_gear.right_hand = Some(next_item);
                commands
                    .entity(next_item)
                    .insert(EquipmentPosition::Hand(Hand::Right));
            }
        }
    }
}

fn assign_received_item_to_slot(
    mut commands: Commands,
    q_new_items: Query<(Entity, Has<GearKind>, Has<Behavior>), Added<LocallyOwned>>,
    mut q_player_gear: Query<(&mut PlayerGear, &Position), With<MainPlayer>>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    let Some((mut player_gear, player_pos)) = q_player_gear.iter_mut().next() else {
        return;
    };

    for (entity, is_gear, is_furniture) in q_new_items.iter() {
        if player_gear.left_hand == Some(entity)
            || player_gear.right_hand == Some(entity)
            || player_gear.inventory.contains(&entity)
            || player_gear
                .held_item
                .as_ref()
                .map(|held| held.entity == entity)
                .unwrap_or(false)
        {
            continue;
        }

        if is_gear {
            if player_gear.left_hand.is_none() {
                player_gear.left_hand = Some(entity);
                commands
                    .entity(entity)
                    .insert(EquipmentPosition::Hand(Hand::Left));
            } else if player_gear.right_hand.is_none() {
                player_gear.right_hand = Some(entity);
                commands
                    .entity(entity)
                    .insert(EquipmentPosition::Hand(Hand::Right));
            } else if player_gear.inventory.len() < 2 {
                player_gear.inventory.push(entity);
                commands.entity(entity).insert(EquipmentPosition::Stowed);
            } else {
                warn!(
                    "auto_equip_replicated_item: received gear {:?} but all slots are full",
                    entity
                );
                continue;
            }
        } else if is_furniture {
            if player_gear.held_item.is_none() {
                player_gear.held_item = Some(HeldObject { entity });
            } else {
                warn!(
                    "auto_equip_replicated_item: received furniture {:?} but held_item is occupied",
                    entity
                );
                continue;
            }
        } else {
            continue;
        }

        ev_sound.write(SoundEvent {
            sound_file: "sounds/item-pickup-whoosh.ogg".to_string(),
            volume: 1.0,
            position: Some(*player_pos),
            broadcast: false,
        });
    }
}

fn strip_visuals_from_grabbed_gear(
    mut commands: Commands,
    mut removed: RemovedComponents<ungear_core::components::deployedgear::DeployedGear>,
    q_gear: Query<(), With<GearMarker>>,
) {
    for entity in removed.read() {
        if q_gear.contains(entity) {
            commands.entity(entity).remove::<(
                Sprite,
                Transform,
                Visibility,
                unrender_std::components::game::GameSprite,
                unrender_std::components::sprite_layer::SpriteLayer,
                unboard_core::components::mapcolor::MapColor,
            )>();
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

fn swap_hand_equipment(
    mut players: Query<(&mut PlayerGear, &PlayerInput)>,
    mut commands: Commands,
) {
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
    use untypes_core::states::AppState;
    app.add_systems(
        Update,
        (
            sync_inventory_position_to_holder,
            sync_held_object_position_to_holder,
        )
            .run_if(in_state(AppState::InGame)),
    );
    app.add_systems(
        Update,
        (
            queue_pickup_request,
            queue_drop_request,
            assign_received_item_to_slot,
            strip_visuals_from_grabbed_gear,
        )
            .run_if(in_state(AppState::InGame)),
    );
    // cycle_inventory and swap_hands are purely local slot rearrangements.
    // They must run on the join client too (not just authority), so they are
    // registered outside PlayerAuthoritativeLogicSet.
    app.add_systems(
        Update,
        (cycle_inventory, swap_hand_equipment).run_if(in_state(AppState::InGame)),
    );
}
