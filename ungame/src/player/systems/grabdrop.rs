use crate::player::{DeployedGear, DeployedGearData, HeldObject, PlayerSprite};
use crate::uncore_board::{self, Position};
use crate::uncore_root;
use bevy::prelude::*;
use uncore::behavior::component::FloorItemCollidable;
use uncore::behavior::Behavior;
use uncore::components::game::GameSprite;
use uncore::systemparam::gear_stuff::GearStuff;
use uncore::traits::gear_usable::GearUsable;
use ungear::components::playergear::PlayerGear;

/// Allows the player to pick up a pickable object from the environment.
///
/// This system checks if the player is pressing the 'grab' key and if there is a
/// pickable object within reach. If so, the object is visually attached to the
/// player, and the player's right-hand gear is disabled. Only one object can be
/// held at a time.
pub fn grab_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite)>,
    deployables: Query<(Entity, &Position), With<DeployedGear>>,
    // Query for all entities with Behavior
    pickables: Query<(Entity, &Position, &Behavior)>,
    mut gs: GearStuff,
) {
    for (mut player_gear, player_pos, player) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.grab) && player_gear.held_item.is_none() {
            // If there's any gear deployed nearby do not consider furniture.
            if deployables
                .iter()
                .any(|(_, object_pos)| player_pos.distance(object_pos) < 1.0)
            {
                return;
            }

            // Find a pickable object near the player
            if let Some((object_entity, _, _)) = pickables
                .iter()
                // Filter for pickable objects
                .filter(|(_, _, behavior)| behavior.p.object.pickable)
                .find(|(_, object_pos, _)| player_pos.distance(object_pos) < 1.0)
            {
                // Set the held object in the player's gear
                player_gear.held_item = Some(HeldObject {
                    entity: object_entity,
                });

                // Play "Pick Up" sound effect
                gs.play_audio("sounds/item-pickup-whoosh.ogg".into(), 1.0, player_pos);
            }
        }
    }
}

/// Allows the player to release a held object back into the environment.
///
/// This system checks if the player is pressing the 'drop' key and if they are
/// currently holding an object. It then determines if the target tile (the
/// player's current position) is a valid drop location (an empty floor tile and
/// not obstructed by other objects).
///
/// If the drop is valid, the object is placed at the target tile. If the drop is
/// invalid, an "invalid drop" sound effect is played, and the object is not
/// dropped.
#[allow(clippy::type_complexity)]
pub fn drop_object(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&mut PlayerGear, &Position, &PlayerSprite), Without<Behavior>>,
    mut objects: Query<(Entity, &mut Position), (Without<PlayerSprite>, With<FloorItemCollidable>)>,
    mut gs: GearStuff,
) {
    for (mut player_gear, player_pos, player) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.drop) {
            // Take the held object from the player's gear (this removes it temporarily)
            if let Some(held_object) = player_gear.held_item.take() {
                // Check for valid Drop location
                let target_tile = player_pos.to_board_position();
                let is_valid_tile = gs
                    .bf
                    .collision_field
                    .get(&target_tile)
                    .map(|col| col.player_free)
                    .unwrap_or(false);

                // Check for object obstruction
                let is_obstructed = objects.iter().any(|(entity, object_pos)| {
                    // Skip checking the held object itself
                    if entity == held_object.entity {
                        return false;
                    }

                    // **Collision Check:**
                    target_tile.to_position().distance(object_pos) < 0.5
                });

                // Only drop if valid
                if is_valid_tile && !is_obstructed {
                    // Retrieve the ORIGINAL entity of the held object
                    if let Ok((_, mut position)) = objects.get_mut(held_object.entity) {
                        // Update the object's Position component
                        *position = target_tile.to_position();

                        // Play "Drop" sound effect
                        gs.play_audio("sounds/item-drop-clunk.ogg".into(), 1.0, player_pos);
                    } else {
                        warn!("Failed to retrieve components from held object entity.");

                        // Put the object back in the player's gear if we can't drop it
                        player_gear.held_item = Some(held_object);
                    }
                } else {
                    // --- Invalid Drop Handling --- Play "Invalid Drop" sound effect
                    gs.play_audio("sounds/invalid-action-buzz.ogg".into(), 0.3, player_pos);

                    // Put the object back in the player's gear
                    player_gear.held_item = Some(held_object);
                }
            }
        }
    }
}

// --- GEAR ----

/// Updates the position of the player's held object to match the player's position.
///
/// This system ensures that the held object visually follows the player when they
/// move. It also slightly elevates the object's Z position to create a visual
/// indication that the object is being held. Additionally, it plays a scraping
/// sound effect when the player moves while holding a movable object, with a
/// cooldown to prevent the sound from playing too frequently.
#[allow(clippy::type_complexity)]
pub fn update_held_object_position(
    mut objects: Query<(&mut Position, &Behavior), Without<PlayerSprite>>,
    players: Query<(&Position, &PlayerGear, &uncore_board::Direction), With<PlayerSprite>>,
    mut gs: GearStuff,
    mut last_sound_time: Local<f32>,
) {
    let current_time = gs.time.elapsed_secs();
    for (player_pos, player_gear, direction) in players.iter() {
        if let Some(held_object) = &player_gear.held_item {
            if let Ok((mut object_pos, behavior)) = objects.get_mut(held_object.entity) {
                // Match the object's position to the player's position
                *object_pos = *player_pos;

                // Slightly elevate the object's Z position
                const OBJECT_ELEVATION: f32 = 0.1;
                object_pos.z += OBJECT_ELEVATION;

                // --- Play Scraping Sound if Object is Movable and Player is Moving ---
                if behavior.p.object.movable
                // Player is moving
                && direction.distance() > 75.0 && current_time - *last_sound_time > 2.0
                // Sound cooldown
                {
                    // Play "Move" sound effect
                    gs.play_audio("sounds/item-move-scrape.ogg".into(), 0.1, player_pos);

                    // Update last sound time
                    *last_sound_time = current_time;
                }
            }
        }
    }
}

/// System for deploying a piece of gear from the player's right hand into the game
/// world.
pub fn deploy_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        &mut PlayerGear,
        &Position,
        &PlayerSprite,
        &uncore_board::Direction,
    )>,
    mut commands: Commands,
    q_collidable: Query<(Entity, &Position), With<FloorItemCollidable>>,
    mut gs: GearStuff,
    handles: Res<uncore_root::GameAssets>,
) {
    for (mut player_gear, player_pos, player, dir) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.drop)
            && player_gear.right_hand.kind.is_some()
            && player_gear.held_item.is_none()
        {
            let deployed_gear = DeployedGear { direction: *dir };
            let target_tile = player_pos.to_board_position();
            let is_valid_tile = gs
                .bf
                .collision_field
                .get(&target_tile)
                .map(|col| col.player_free)
                .unwrap_or(false);
            let is_obstructed = q_collidable
                .iter()
                .any(|(_entity, object_pos)| target_tile.to_position().distance(object_pos) < 0.5);
            if is_valid_tile && !is_obstructed {
                let scoord = player_pos.to_screen_coord();
                let gear_sprite = Sprite {
                    image: handles.images.gear.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: handles.images.gear_atlas.clone(),
                        index: player_gear.right_hand.get_sprite_idx() as usize,
                    }),
                    ..Default::default()
                };
                commands
                    .spawn(gear_sprite)
                    .insert(
                        // Initial scaling factor
                        Transform::from_xyz(scoord.x, scoord.y, scoord.z + 0.01)
                            .with_scale(Vec3::new(0.25, 0.25, 0.25)),
                    )
                    .insert(deployed_gear)
                    .insert(*player_pos)
                    .insert(FloorItemCollidable)
                    .insert(GameSprite)
                    .insert(DeployedGearData {
                        gear: player_gear.right_hand.take(),
                    });
                player_gear.cycle();

                // Play "Drop Item" sound effect (reused for gear deployment)
                gs.play_audio("sounds/item-drop-clunk.ogg".into(), 1.0, player_pos);
            } else {
                // Play "Invalid Drop" sound effect
                gs.play_audio("sounds/invalid-action-buzz.ogg".into(), 0.3, player_pos);
            }
        }
    }
}

/// System for retrieving deployed gear and adding it to the player's right hand.
pub fn retrieve_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&Position, &PlayerSprite, &mut PlayerGear)>,
    q_deployed: Query<(Entity, &Position, &DeployedGearData)>,
    mut commands: Commands,
    mut gs: GearStuff,
) {
    // FIXME: This code, along with grabbing items are in conflict. It will be
    // possible for a player to grab equipment from the floor and a location item at
    // the same time if they are close enough for a well placed player. This needs to
    // be solved, likely by handling the keypress event in one single system, then
    // routing the remaining stuff to do via an Event to the system that handles that
    // exact thing.
    for (player_pos, player, mut player_gear) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.grab) {
            // Find the closest deployed gear
            let mut closest_gear: Option<(Entity, f32)> = None;
            for (entity, gear_pos, _) in q_deployed.iter() {
                let distance = player_pos.distance(gear_pos);
                if distance < 1.2 {
                    if let Some((_, closest_distance)) = closest_gear {
                        if distance < closest_distance {
                            closest_gear = Some((entity, distance));
                        }
                    } else {
                        closest_gear = Some((entity, distance));
                    }
                }
            }

            // Retrieve the closest gear
            if let Some((closest_gear_entity, _)) = closest_gear {
                if let Ok((_, _, deployed_gear_data)) = q_deployed.get(closest_gear_entity) {
                    // Inventory Shifting Logic:
                    if player_gear.right_hand.kind.is_some() {
                        // Right hand is occupied, try to shift to inventory
                        if let Some(empty_slot_index) = player_gear
                            .inventory
                            .iter()
                            .position(|gear| gear.kind.is_none())
                        {
                            // Move right-hand gear to the empty slot
                            player_gear.inventory[empty_slot_index] = player_gear.right_hand.take();
                        } else {
                            // No empty slot - play invalid action sound and skip retrieval
                            gs.play_audio("sounds/invalid-action-buzz.ogg".into(), 0.3, player_pos);
                            return;
                        }
                    }

                    // Now the right hand is free, proceed with retrieval
                    player_gear.right_hand = deployed_gear_data.gear.clone();
                    commands.entity(closest_gear_entity).despawn();

                    // Play "Grab Item" sound effect (reused for gear retrieval)
                    gs.play_audio("sounds/item-pickup-whoosh.ogg".into(), 1.0, player_pos);
                }
            }
            // --
        }
    }
}
