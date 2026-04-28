use bevy::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::components::FloorItemCollidable;
use unboard_core::entity::GameSprite;
use unboard_core::resources::board_topology::BoardCollisionField;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::equipment::{EquipmentPosition, Hand};
use ungear_core::types::gear::kind::GearKind;
use uninput_core::components::PlayerInput;
use unlocomotion_core::components::PlayerLocomotionState;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unreplicon_core::components::SimulationAuthorized;
use unreplicon_core::messages::{RequestDrop, RequestGrab};
use unreplicon_core::ownership::LocallyOwned;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

pub(crate) fn sync_inventory_position_to_holder(
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

pub(crate) fn sync_held_object_position_to_holder(
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

pub(crate) fn queue_pickup_request(
    mut players: Query<(&PlayerGear, &Position, &mut PlayerInput), With<MainPlayer>>,
    pickables: Query<
        (Entity, &Position, Option<&GearKind>, Option<&Behavior>),
        (Without<PlayerSprite>, With<FloorItemCollidable>),
    >,
    mut writer_grab: MessageWriter<RequestGrab>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    for (player_gear, player_pos, mut player_input) in players.iter_mut() {
        if player_input.grab {
            player_input.grab = false;
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
                    writer_grab.write(RequestGrab { entity });
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

pub(crate) fn queue_drop_request(
    mut players: Query<(
        &mut PlayerGear,
        &Position,
        &mut PlayerInput,
        &PlayerLocomotionState,
    )>,
    mut commands: Commands,
    board_collision: Res<BoardCollisionField>,
    pickables: Query<&Position, (With<FloorItemCollidable>, Without<PlayerSprite>)>,
    mut writer_drop: MessageWriter<RequestDrop>,
    mut ev_sound: MessageWriter<SoundEvent>,
) {
    for (mut player_gear, player_pos, mut player_input, player_loco) in players.iter_mut() {
        if player_input.drop {
            player_input.drop = false;
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

            // Remove LocallyOwned - the server dictates inventory slots.
            commands.entity(entity).remove::<LocallyOwned>();

            let drop_direction = Direction {
                dx: player_loco.movement.dx,
                dy: player_loco.movement.dy,
                dz: player_loco.movement.dz,
            };
            // Use the player's actual z (floor level) for both gear and furniture.
            let drop_pos = Position {
                x: player_pos.x,
                y: player_pos.y,
                z: player_pos.z,
                ..*player_pos
            };
            commands
                .entity(entity)
                .insert((FloorItemCollidable, drop_pos, drop_direction));
            if dropped_gear {
                commands.entity(entity).insert((
                    DeployedGear {
                        direction: drop_direction,
                    },
                    EquipmentPosition::Deployed,
                ));
            }
            writer_drop.write(RequestDrop {
                entity,
                position: [player_pos.x, player_pos.y, player_pos.z],
                direction: [
                    player_loco.movement.dx,
                    player_loco.movement.dy,
                    player_loco.movement.dz,
                ],
            });
            ev_sound.write(SoundEvent {
                sound_file: "sounds/item-drop-clunk.ogg".to_string(),
                volume: 1.0,
                position: Some(*player_pos),
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

pub(crate) fn strip_visuals_from_grabbed_gear(
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
                GameSprite,
                unrender_std::components::sprite_layer::SpriteLayer,
                unboard_core::components::mapcolor::MapColor,
            )>();
        }
    }
}

pub(crate) fn cycle_inventory(
    mut players: Query<(&mut PlayerGear, &mut PlayerInput)>,
    mut commands: Commands,
) {
    for (mut player_gear, mut player_input) in players.iter_mut() {
        if player_input.inventory_cycle {
            player_input.inventory_cycle = false;
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

        if player_input.inventory_cycle_prev {
            player_input.inventory_cycle_prev = false;
            if let Some(entity) = player_gear.right_hand.take() {
                player_gear.inventory.insert(0, entity);
                commands.entity(entity).insert(EquipmentPosition::Stowed);
            }
            if let Some(entity) = player_gear.inventory.pop() {
                player_gear.right_hand = Some(entity);
                commands
                    .entity(entity)
                    .insert(EquipmentPosition::Hand(Hand::Right));
            }
        }
    }
}

pub(crate) fn swap_hand_equipment(
    mut players: Query<(&mut PlayerGear, &mut PlayerInput)>,
    mut commands: Commands,
) {
    for (mut player_gear, mut player_input) in players.iter_mut() {
        if player_input.inventory_swap {
            player_input.inventory_swap = false;
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

/// When a player dies (PlayerDiedEvent fires), despawn all their gear (Authoritative only).
/// This moves the gear cleanup responsibility from the Vitals domain to the Inventory domain.
pub(crate) fn despawn_gear_on_player_death(
    mut reader: MessageReader<unvitals_core::events::PlayerDiedEvent>,
    mut q_players: Query<(&PlayerSprite, &mut PlayerGear)>,
    mut commands: Commands,
    authority: Option<Res<unreplicon_core::resources::AuthorityRole>>,
) {
    if authority.is_none() {
        return;
    }

    for msg in reader.read() {
        // When a player dies, despawn all their gear (Authoritative only)
        for (sprite, mut gear) in q_players.iter_mut() {
            if sprite.network_id == msg.id {
                if let Some(e) = gear.left_hand {
                    commands.entity(e).despawn();
                }
                if let Some(e) = gear.right_hand {
                    commands.entity(e).despawn();
                }
                for e in gear.inventory.iter() {
                    commands.entity(*e).despawn();
                }
                if let Some(h) = &gear.held_item {
                    commands.entity(h.entity).despawn();
                }
                // Empty the inventory
                *gear = PlayerGear::default();
            }
        }
    }
}

fn client_sync_inventory_state(
    q_player: Query<&PlayerGear, (With<LocallyOwned>, Changed<PlayerGear>)>,
    mut commands: Commands,
) {
    // NOTE: we filter by "Changed" because otherwise we would be constantly rewritting and on top of that if we want
    // to despawn the entities here it would have been receiving commands constantly.
    for gear in q_player.iter() {
        if let Some(e) = gear.left_hand {
            commands.entity(e).insert((
                EquipmentPosition::Hand(Hand::Left),
                LocallyOwned,
                SimulationAuthorized,
            ));
        }
        if let Some(e) = gear.right_hand {
            commands.entity(e).insert((
                EquipmentPosition::Hand(Hand::Right),
                LocallyOwned,
                SimulationAuthorized,
            ));
        }
        for &e in &gear.inventory {
            commands.entity(e).insert((
                EquipmentPosition::Stowed,
                LocallyOwned,
                SimulationAuthorized,
            ));
        }
        if let Some(held) = &gear.held_item {
            commands
                .entity(held.entity)
                .insert((LocallyOwned, SimulationAuthorized));
        }
    }
}

fn client_cleanup_orphaned_shields(
    q_shielded: Query<Entity, (With<LocallyOwned>, Or<(With<GearMarker>, With<Behavior>)>)>,
    q_player: Query<&PlayerGear, With<LocallyOwned>>,
    mut commands: Commands,
) {
    let Ok(gear) = q_player.single() else {
        return;
    };

    // Build a list of everything actually in our pockets
    let mut in_pockets = vec![];
    if let Some(e) = gear.left_hand {
        in_pockets.push(e);
    }
    if let Some(e) = gear.right_hand {
        in_pockets.push(e);
    }
    in_pockets.extend(gear.inventory.iter());
    if let Some(held) = &gear.held_item {
        in_pockets.push(held.entity);
    }

    // If an item has our shield but isn't in our pockets, strip the shield
    for entity in q_shielded.iter() {
        if !in_pockets.contains(&entity) {
            commands.entity(entity).remove::<LocallyOwned>();
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    use uncommon_states_core::UIContextState;
    app.add_systems(
        Update,
        (
            sync_inventory_position_to_holder,
            sync_held_object_position_to_holder,
        )
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        Update,
        (
            queue_pickup_request,
            queue_drop_request,
            cycle_inventory,
            swap_hand_equipment,
        )
            .after(uninput_core::PlayerInputSet)
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        Update,
        (
            client_sync_inventory_state,
            client_cleanup_orphaned_shields,
            strip_visuals_from_grabbed_gear,
            despawn_gear_on_player_death,
        )
            .run_if(in_state(UIContextState::InGame)),
    );
}
