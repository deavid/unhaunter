use bevy::prelude::*;
use bevy_replicon::prelude::{
    Channel, ClientId, ClientMessageAppExt, FromClient, Replicated, SendMode, ToClients,
};
use unbehavior_core::behavior::Behavior;
use unbehavior_core::components::FloorItemCollidable;
use uncommon_app_core::random_seed;
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::difficulty_ext::DifficultyGearExt;
use ungear_core::events::{
    RequestEquipGearFromVan, RequestUnequipHand, RequestUnequipInventorySlot,
};
use ungear_core::messages::{ExportPlayerGearMessage, TruckLoadoutAction, TruckLoadoutMessage};
use ungear_core::resources::spawner::{GearHydrated, GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::equipment::{EquipmentPosition, Hand};
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::repellentflask::RepellentFlask;
use unmission_core::types::SimulationState;
use unplayer_core::components::{MainPlayer, PlayerDisconnected, PlayerSprite};
use unreplicon_core::components::NetworkEntityReady;
use unreplicon_core::events::{PlayerNetworkDisconnected, PlayerNetworkReconnected};
use unreplicon_core::messages::{OwnershipGranted, OwnershipRevoked, RequestDrop, RequestGrab};
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole, is_pure_client};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

fn to_owner_id(client_id: ClientId) -> OwnerId {
    match client_id {
        ClientId::Server => OwnerId::Server,
        ClientId::Client(e) => OwnerId::Client(e),
    }
}

fn gear_snapshot(gear: &PlayerGear) -> String {
    format!(
        "left={:?} right={:?} inv={:?} held={:?}",
        gear.left_hand, gear.right_hand, gear.inventory, gear.held_item
    )
}

fn send_export_player_gear(
    q_local: Query<(&PlayerGear, Has<InTruck>), (With<LocallyOwned>, With<PlayerSprite>)>,
    mut writer: MessageWriter<ExportPlayerGearMessage>,
    mut last_export_snapshot: Local<Option<String>>,
    mut export_suppressed: Local<bool>,
) {
    for (gear, in_truck) in q_local.iter() {
        if in_truck {
            if !*export_suppressed {
                info!(
                    "PLAYER_GEAR_EXPORT_CLIENT: suppressing export while local player is in truck"
                );
                *export_suppressed = true;
            }
            continue;
        }

        if *export_suppressed {
            info!("PLAYER_GEAR_EXPORT_CLIENT: resuming export after leaving truck");
            *export_suppressed = false;
        }

        let snapshot = gear_snapshot(gear);
        if last_export_snapshot.as_ref() != Some(&snapshot) {
            info!(
                "PLAYER_GEAR_EXPORT_CLIENT: exporting locally owned gear state {}",
                snapshot
            );
            *last_export_snapshot = Some(snapshot);

            writer.write(ExportPlayerGearMessage {
                left_hand: gear.left_hand,
                right_hand: gear.right_hand,
                inventory: gear.inventory.clone(),
                held_item: gear.held_item.clone(),
            });
        }
    }
}

fn handle_export_player_gear_state(
    mut reader: MessageReader<FromClient<ExportPlayerGearMessage>>,
    mut q_players: Query<(&Owner, &mut PlayerGear, Has<InTruck>), Without<LocallyOwned>>,
) {
    // TODO: Theoretical race condition: if an unreliable ExportPlayerGearMessage arrives
    // out-of-order AFTER a RequestDrop has been processed, the server might briefly put
    // the dropped item back into the player's inventory.
    for msg in reader.read() {
        let mut found_player = false;
        for (owner, mut gear, in_truck) in q_players.iter_mut() {
            if from_owner_id(owner.0) == msg.client_id {
                found_player = true;
                if in_truck {
                    info!(
                        "PLAYER_GEAR_EXPORT_SERVER: ignoring client {:?} export while authoritative player is in truck",
                        msg.client_id
                    );
                    break;
                }
                let before_snapshot = gear_snapshot(&gear);
                gear.left_hand = msg.message.left_hand;
                gear.right_hand = msg.message.right_hand;
                gear.inventory = msg.message.inventory.clone();
                gear.held_item = msg.message.held_item.clone();
                let after_snapshot = gear_snapshot(&gear);
                if before_snapshot != after_snapshot {
                    info!(
                        "PLAYER_GEAR_EXPORT_SERVER: applied client {:?} export; before={} after={}",
                        msg.client_id, before_snapshot, after_snapshot
                    );
                }
                break;
            }
        }
        if !found_player {
            warn!(
                "PLAYER_GEAR_EXPORT_SERVER: no PlayerGear found for client {:?}; export dropped",
                msg.client_id
            );
        }
    }
}

fn handle_request_grab(
    mut reader: MessageReader<FromClient<RequestGrab>>,
    mut commands: Commands,
    q_items: Query<
        (Entity, Option<&Owner>, Has<GearKind>, Has<Behavior>),
        With<FloorItemCollidable>,
    >,
) {
    for msg in reader.read() {
        let client_id = msg.client_id;
        let item_entity = msg.message.entity;

        if let Ok((entity, old_owner, is_gear, is_furniture)) = q_items.get(item_entity) {
            // Revoke simulation authority from the old driver if different from the new grabber.
            if let Some(old_owner) = old_owner {
                let old_client_id = from_owner_id(old_owner.0);
                if old_client_id != client_id {
                    if old_client_id == ClientId::Server {
                        commands.entity(entity).remove::<LocallyOwned>();
                    } else {
                        commands.write_message(ToClients {
                            mode: SendMode::Direct(old_client_id),
                            message: OwnershipRevoked { entity },
                        });
                    }
                }
            }

            let owner_id = to_owner_id(client_id);
            commands.entity(entity).insert(Owner(owner_id));
            commands.entity(entity).remove::<FloorItemCollidable>();
            commands.entity(entity).remove::<DeployedGear>();

            if is_gear {
                // Gear visual cleanup is handled reactively on clients.
            }

            if is_furniture {
                // Furniture keeps its visuals while carried.
            }

            if client_id == ClientId::Server {
                commands.entity(entity).insert(LocallyOwned);
            } else {
                commands.write_message(ToClients {
                    mode: SendMode::Direct(client_id),
                    message: OwnershipGranted { entity },
                });
            }
        }
    }
}

fn handle_request_drop(
    mut reader: MessageReader<FromClient<RequestDrop>>,
    mut commands: Commands,
    q_items: Query<(Entity, &Owner, Has<GearKind>, Has<Behavior>)>,
) {
    for msg in reader.read() {
        let client_id = msg.client_id;
        let item_entity = msg.message.entity;

        if let Ok((entity, owner, is_gear, is_furniture)) = q_items.get(item_entity)
            && from_owner_id(owner.0) == client_id
        {
            // Owner is intentionally retained — the dropping player remains the Designated Driver
            // and continues simulating the gear's internal state while it is on the floor.
            commands.entity(entity).insert(FloorItemCollidable);
            commands.entity(entity).insert(Position {
                x: msg.message.position[0],
                y: msg.message.position[1],
                z: msg.message.position[2],
                ..Default::default()
            });

            if is_gear {
                commands.entity(entity).insert(DeployedGear {
                    direction: Direction {
                        dx: msg.message.direction[0],
                        dy: msg.message.direction[1],
                        dz: msg.message.direction[2],
                    },
                });
                commands.entity(entity).insert(EquipmentPosition::Deployed);
            }

            if is_furniture {
                // Furniture visuals are preserved while carried; no extra components needed.
            }
        }
    }
}

fn handle_truck_loadout_message(
    mut reader: MessageReader<FromClient<TruckLoadoutMessage>>,
    mut q_players: Query<(&Owner, &mut PlayerGear)>,
    gear_registry: Res<GearSpawnerRegistry>,
    q_gearkind: Query<&GearKind>,
    mut q_repellent: Query<&mut RepellentFlask>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let sender_id = msg.client_id;
        info!(
            "TRUCK_NET: Received action {:?} from client {:?}",
            msg.message.action, sender_id
        );

        let Some((owner, mut p_gear)) = q_players.iter_mut().find_map(|(owner, gear)| {
            if from_owner_id(owner.0) == sender_id {
                Some((owner, gear))
            } else {
                None
            }
        }) else {
            warn!(
                "TRUCK_NET_FAIL: Could not find PlayerGear component for client {:?}",
                sender_id
            );
            continue;
        };

        info!(
            "TRUCK_NET_STATE_BEFORE: client {:?} left={:?} right={:?} inv={:?}",
            sender_id, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
        );

        match msg.message.action {
            TruckLoadoutAction::AddGear(kind) => {
                let has_space = p_gear.left_hand.is_none()
                    || p_gear.right_hand.is_none()
                    || p_gear.inventory.len() < 2;
                if !has_space {
                    warn!(
                        "TRUCK_NET_FAIL: Client {:?} AddGear but inventory is full on server!",
                        sender_id
                    );
                    continue;
                }
                let entity = gear_registry.spawn(&mut commands, kind);
                let rng_val = random_seed::heavy_rng_seed();
                let net_id = NetworkId(rng_val.max(1000));

                commands
                    .entity(entity)
                    .insert((Replicated, Owner(owner.0), net_id));

                let client_id = from_owner_id(owner.0);
                if client_id != ClientId::Server {
                    commands.write_message(ToClients {
                        mode: SendMode::Direct(client_id),
                        message: OwnershipGranted { entity },
                    });
                }

                if p_gear.left_hand.is_none() {
                    p_gear.left_hand = Some(entity);
                } else if p_gear.right_hand.is_none() {
                    p_gear.right_hand = Some(entity);
                } else {
                    p_gear.inventory.push(entity);
                }
                info!(
                    "TRUCK_NET: Spawned {:?} and added to PlayerGear for {:?}",
                    entity, sender_id
                );
                info!(
                    "TRUCK_NET_STATE_AFTER: client {:?} left={:?} right={:?} inv={:?}",
                    sender_id, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
                );
            }
            TruckLoadoutAction::CraftRepellent(ghost_type) => {
                debug!(
                    "REPELLENT: Received CraftRepellent({:?}) from client {:?}",
                    ghost_type, sender_id
                );

                let mut flask_entity: Option<Entity> = None;

                if let Some(entity) = p_gear.right_hand {
                    match q_gearkind.get(entity) {
                        Ok(kind) if *kind == GearKind::RepellentFlask => {
                            debug!(
                                "REPELLENT: found existing server-side flask in right hand entity {:?}",
                                entity
                            );
                            flask_entity = Some(entity);
                        }
                        Ok(kind) => {
                            debug!(
                                "REPELLENT: server right hand entity {:?} is {:?}, not RepellentFlask",
                                entity, kind
                            );
                        }
                        Err(err) => {
                            warn!(
                                "REPELLENT: failed to read GearKind for server right hand entity {:?}: {}",
                                entity, err
                            );
                        }
                    }
                }

                if flask_entity.is_none()
                    && let Some(entity) = p_gear.left_hand
                {
                    match q_gearkind.get(entity) {
                        Ok(kind) if *kind == GearKind::RepellentFlask => {
                            debug!(
                                "REPELLENT: found existing server-side flask in left hand entity {:?}",
                                entity
                            );
                            flask_entity = Some(entity);
                        }
                        Ok(kind) => {
                            debug!(
                                "REPELLENT: server left hand entity {:?} is {:?}, not RepellentFlask",
                                entity, kind
                            );
                        }
                        Err(err) => {
                            warn!(
                                "REPELLENT: failed to read GearKind for server left hand entity {:?}: {}",
                                entity, err
                            );
                        }
                    }
                }

                if flask_entity.is_none() {
                    for &entity in &p_gear.inventory {
                        match q_gearkind.get(entity) {
                            Ok(kind) if *kind == GearKind::RepellentFlask => {
                                debug!(
                                    "REPELLENT: found existing server-side flask in inventory entity {:?}",
                                    entity
                                );
                                flask_entity = Some(entity);
                                break;
                            }
                            Ok(kind) => {
                                debug!(
                                    "REPELLENT: server inventory entity {:?} is {:?}, not RepellentFlask",
                                    entity, kind
                                );
                            }
                            Err(err) => {
                                warn!(
                                    "REPELLENT: failed to read GearKind for server inventory entity {:?}: {}",
                                    entity, err
                                );
                            }
                        }
                    }
                }

                let is_new = flask_entity.is_none();
                if is_new {
                    let entity = gear_registry.spawn(&mut commands, GearKind::RepellentFlask);
                    let rng_val = uncommon_app_core::random_seed::heavy_rng_seed();
                    let net_id = NetworkId(rng_val.max(1000));

                    debug!(
                        "REPELLENT: spawning server-side RepellentFlask entity {:?} net_id={:?} for client {:?}",
                        entity, net_id, sender_id
                    );

                    commands
                        .entity(entity)
                        .insert((Replicated, Owner(owner.0), net_id));

                    let client_id = from_owner_id(owner.0);
                    if client_id != ClientId::Server {
                        commands.write_message(ToClients {
                            mode: SendMode::Direct(client_id),
                            message: OwnershipGranted { entity },
                        });
                    }

                    if let Some(old_rh) = p_gear.right_hand.take() {
                        if p_gear.inventory.len() < 2 {
                            debug!(
                                "REPELLENT: moving previous server right hand entity {:?} into inventory for client {:?}",
                                old_rh, sender_id
                            );
                            p_gear.inventory.push(old_rh);
                        } else {
                            warn!(
                                "REPELLENT: inventory full while crafting for client {:?}; despawning previous right hand entity {:?}",
                                sender_id, old_rh
                            );
                            commands.entity(old_rh).despawn();
                        }
                    }

                    p_gear.right_hand = Some(entity);
                    flask_entity = Some(entity);
                }

                let Some(entity) = flask_entity else {
                    error!(
                        "REPELLENT: flask_entity is None after search and spawn for client {:?}",
                        sender_id
                    );
                    continue;
                };

                if is_new {
                    debug!(
                        "REPELLENT: inserting crafted flask state on new entity {:?} ghost_type={:?} for client {:?}",
                        entity, ghost_type, sender_id
                    );
                    commands.entity(entity).insert(RepellentFlask {
                        liquid_content: Some(ghost_type),
                        qty: RepellentFlask::MAX_QTY,
                        active: false,
                    });
                } else if let Ok(mut flask) = q_repellent.get_mut(entity) {
                    debug!(
                        "REPELLENT: refilling existing server-side flask entity {:?} with ghost_type={:?} for client {:?}",
                        entity, ghost_type, sender_id
                    );
                    flask.liquid_content = Some(ghost_type);
                    flask.qty = RepellentFlask::MAX_QTY;
                    flask.active = false;
                } else {
                    error!(
                        "REPELLENT: failed to get RepellentFlask on entity {:?} for client {:?}",
                        entity, sender_id
                    );
                    continue;
                }

                info!(
                    "REPELLENT: crafted/refilled repellent for client {:?}; left={:?} right={:?} inv={:?}",
                    sender_id, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
                );
            }
            TruckLoadoutAction::ClearHand(hand) => {
                let entity = match hand {
                    Hand::Left => p_gear.left_hand.take(),
                    Hand::Right => p_gear.right_hand.take(),
                };
                if let Some(e) = entity {
                    info!("TRUCK_NET: Despawning entity {:?} from hand {:?}", e, hand);
                    commands.entity(e).despawn();
                    info!(
                        "TRUCK_NET_STATE_AFTER: client {:?} left={:?} right={:?} inv={:?}",
                        sender_id, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
                    );
                } else {
                    warn!(
                        "TRUCK_NET_FAIL: Client {:?} requested ClearHand({:?}) but server's PlayerGear slot was ALREADY EMPTY!",
                        sender_id, hand
                    );
                    warn!(
                        "TRUCK_NET_STATE: Server thinks gear is -> left: {:?}, right: {:?}, inv: {:?}",
                        p_gear.left_hand, p_gear.right_hand, p_gear.inventory
                    );
                }
            }
            TruckLoadoutAction::ClearInventorySlot(idx) => {
                if idx < p_gear.inventory.len() {
                    let e = p_gear.inventory.remove(idx);
                    info!(
                        "TRUCK_NET: Despawning entity {:?} from inventory slot {}",
                        e, idx
                    );
                    commands.entity(e).despawn();
                    info!(
                        "TRUCK_NET_STATE_AFTER: client {:?} left={:?} right={:?} inv={:?}",
                        sender_id, p_gear.left_hand, p_gear.right_hand, p_gear.inventory
                    );
                } else {
                    warn!(
                        "TRUCK_NET_FAIL: Client {:?} requested ClearInventorySlot({}), but server's inventory length is {}",
                        sender_id,
                        idx,
                        p_gear.inventory.len()
                    );
                }
            }
        }
    }
}

fn orphan_catcher(
    mut reader: MessageReader<PlayerNetworkDisconnected>,
    q_all_players: Query<(&PlayerSprite, &Owner)>,
    q_eligible_survivors: Query<
        (&PlayerSprite, &Owner),
        (With<NetworkEntityReady>, Without<PlayerDisconnected>),
    >,
    mut q_gear: Query<(Entity, &mut Owner), Without<PlayerSprite>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Identify the disconnected player's OwnerId (may already have PlayerDisconnected,
        // so we search all players to reliably find their OwnerId).
        let Some(disconnected_owner) = q_all_players
            .iter()
            .find(|(sprite, _)| sprite.id == msg.player_uuid)
            .map(|(_, owner)| owner.0)
        else {
            warn!(
                "orphan_catcher: no player entity found for UUID {}",
                msg.player_uuid
            );
            continue;
        };

        // Find a surviving, ready, connected player to inherit simulation authority.
        let Some(survivor_owner) = q_eligible_survivors
            .iter()
            .find(|(sprite, _)| sprite.id != msg.player_uuid)
            .map(|(_, owner)| owner.0)
        else {
            warn!(
                "orphan_catcher: no surviving player for orphaned gear of UUID {}",
                msg.player_uuid
            );
            continue;
        };

        let survivor_client_id = from_owner_id(survivor_owner);

        // Reassign all gear (inventory or deployed) owned by the disconnected player.
        for (entity, mut owner) in q_gear.iter_mut() {
            if owner.0 != disconnected_owner {
                continue;
            }
            owner.0 = survivor_owner;
            if survivor_client_id == ClientId::Server {
                commands.entity(entity).insert(LocallyOwned);
            } else {
                commands.write_message(ToClients {
                    mode: SendMode::Direct(survivor_client_id),
                    message: OwnershipGranted { entity },
                });
            }
            info!(
                "orphan_catcher: reassigned entity {:?} from {:?} to {:?}",
                entity, disconnected_owner, survivor_owner
            );
        }
    }
}

fn reconcile_gear_on_reconnect(
    mut reader: MessageReader<PlayerNetworkReconnected>,
    q_players: Query<(&PlayerSprite, &PlayerGear)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Some((_sprite, gear)) = q_players
            .iter()
            .find(|(sprite, _)| sprite.id == msg.player_uuid)
        else {
            warn!(
                "reconcile_gear_on_reconnect: no player found for UUID {}",
                msg.player_uuid
            );
            continue;
        };

        for gear_entity in gear
            .left_hand
            .into_iter()
            .chain(gear.right_hand)
            .chain(gear.inventory.iter().copied())
        {
            commands.entity(gear_entity).insert(Owner(msg.new_owner_id));
        }
    }
}

fn hydrate_gear_system(
    mut commands: Commands,
    gear_registry: Res<GearSpawnerRegistry>,
    q_added: Query<(Entity, &GearKind), (With<GearMarker>, Without<GearHydrated>)>,
) {
    for (entity, kind) in q_added.iter() {
        gear_registry.hydrate(&mut commands, entity, *kind);
        commands.entity(entity).insert(GearHydrated);
        info!(
            "hydrate_gear_system: hydrated gear entity {:?} kind={:?}",
            entity, kind
        );
    }
}

/// Authority: spawns gear entities and inserts PlayerGear for any player entity that is
/// missing it. Reacts to PlayerSprite entities added by the network layer (setup_mission_players
/// and spawn_late_joining_players). Uses PlayerSprite.network_id and Owner already on the
/// player entity so no LobbyInfo lookup is required.
pub(crate) fn hydrate_player_gear(
    mut commands: Commands,
    q_new: Query<(Entity, &PlayerSprite, Option<&LocallyOwned>, &Owner), Without<PlayerGear>>,
    difficulty: Res<CurrentDifficulty>,
    gear_registry: Res<GearSpawnerRegistry>,
) {
    for (entity, player_sprite, locally_owned, owner) in q_new.iter() {
        let gear_owner_id = owner.0;
        let mut gear_id_counter = (player_sprite.network_id.0 % 1_000_000) * 1000;

        let player_gear_loadout = difficulty.0.player_gear();
        let mut player_gear = PlayerGear::default();
        let mut gear_entities = Vec::new();

        if player_gear_loadout.left_hand.is_some() {
            let gear_entity = gear_registry.spawn(&mut commands, player_gear_loadout.left_hand);
            player_gear.left_hand = Some(gear_entity);
            gear_entities.push(gear_entity);
            commands.entity(gear_entity).insert((
                NetworkId(gear_id_counter),
                Replicated,
                Owner(gear_owner_id),
            ));
            gear_id_counter += 1;
        }
        if player_gear_loadout.right_hand.is_some() {
            let gear_entity = gear_registry.spawn(&mut commands, player_gear_loadout.right_hand);
            player_gear.right_hand = Some(gear_entity);
            gear_entities.push(gear_entity);
            commands.entity(gear_entity).insert((
                NetworkId(gear_id_counter),
                Replicated,
                Owner(gear_owner_id),
            ));
            gear_id_counter += 1;
        }
        for kind in &player_gear_loadout.inventory {
            if kind.is_some() {
                let gear_entity = gear_registry.spawn(&mut commands, *kind);
                player_gear.inventory.push(gear_entity);
                gear_entities.push(gear_entity);
                commands.entity(gear_entity).insert((
                    NetworkId(gear_id_counter),
                    Replicated,
                    Owner(gear_owner_id),
                ));
                gear_id_counter += 1;
            }
        }

        commands.entity(entity).insert(player_gear);

        // Fulfill the readiness contract: signal to the network layer that this
        // entity is safe to hand over to the client.
        commands
            .entity(entity)
            .insert(unreplicon_core::components::NetworkEntityReady);

        // Propagate LocallyOwned to gear if the player entity is locally owned.
        if locally_owned.is_some() {
            for &gear_entity in &gear_entities {
                commands.entity(gear_entity).insert(LocallyOwned);
            }
        }

        info!(
            "hydrate_player_gear: spawned {} gear entities for player {:?} (owner={:?})",
            gear_entities.len(),
            entity,
            gear_owner_id
        );
    }
}

/// Authority-side handler for truck loadout intent messages emitted by `untruck-plugin`.
///
/// Handles `RequestEquipGearFromVan`, `RequestUnequipHand`, and `RequestUnequipInventorySlot`
/// for the local (authority) player. Join-client loadout requests are handled by the
/// replicated `TruckLoadoutMessage` handler in this crate as well.
pub(crate) fn handle_truck_loadout_request(
    mut ev_equip_van: MessageReader<RequestEquipGearFromVan>,
    mut ev_unequip_hand: MessageReader<RequestUnequipHand>,
    mut ev_unequip_slot: MessageReader<RequestUnequipInventorySlot>,
    mut q_gear: Query<&mut PlayerGear, With<MainPlayer>>,
    gear_registry: Res<GearSpawnerRegistry>,
    mut commands: Commands,
) {
    let Ok(mut p_gear) = q_gear.single_mut() else {
        // No local MainPlayer: dedicated server mode or pre-spawn. Consume and discard events.
        for _ in ev_equip_van.read() {}
        for _ in ev_unequip_hand.read() {}
        for _ in ev_unequip_slot.read() {}
        return;
    };

    for ev in ev_equip_van.read() {
        let has_space =
            p_gear.left_hand.is_none() || p_gear.right_hand.is_none() || p_gear.inventory.len() < 2;
        if !has_space {
            warn!(
                "handle_truck_loadout_request: RequestEquipGearFromVan({:?}) but inventory is full",
                ev.kind
            );
            continue;
        }
        let entity = gear_registry.spawn(&mut commands, ev.kind);
        let rng_val = random_seed::heavy_rng_seed();
        let net_id = NetworkId(rng_val.max(1000));
        commands
            .entity(entity)
            .insert((net_id, Replicated, Owner(OwnerId::Server), LocallyOwned));

        if p_gear.left_hand.is_none() {
            p_gear.left_hand = Some(entity);
        } else if p_gear.right_hand.is_none() {
            p_gear.right_hand = Some(entity);
        } else {
            p_gear.inventory.push(entity);
        }
    }

    for ev in ev_unequip_hand.read() {
        let entity = match ev.hand {
            Hand::Left => p_gear.left_hand.take(),
            Hand::Right => p_gear.right_hand.take(),
        };
        if let Some(e) = entity {
            commands.entity(e).despawn();
        } else {
            warn!(
                "handle_truck_loadout_request: RequestUnequipHand({:?}) but slot is already empty",
                ev.hand
            );
        }
    }

    for ev in ev_unequip_slot.read() {
        if ev.idx >= p_gear.inventory.len() {
            warn!(
                "handle_truck_loadout_request: RequestUnequipInventorySlot({}) out of bounds (len={})",
                ev.idx,
                p_gear.inventory.len()
            );
            continue;
        }
        let e = p_gear.inventory.remove(ev.idx);
        commands.entity(e).despawn();
    }
}

/// Authority-owned teardown for gear entities.
pub(crate) fn despawn_gear_on_teardown(
    mut commands: Commands,
    q_gear: Query<Entity, With<GearMarker>>,
) {
    for entity in q_gear.iter() {
        commands.entity(entity).despawn();
    }
}

fn propagate_gear_ownership(
    mut commands: Commands,
    player_q: Query<(&PlayerGear, &LocallyOwned)>,
    gear_q: Query<Entity, (With<GearMarker>, Without<LocallyOwned>)>,
) {
    for (player_gear, _locally_owned) in player_q.iter() {
        let gear_entities = player_gear
            .left_hand
            .into_iter()
            .chain(player_gear.right_hand)
            .chain(player_gear.inventory.iter().copied());

        for gear_entity in gear_entities {
            if gear_q.get(gear_entity).is_ok() {
                commands.entity(gear_entity).insert(LocallyOwned);
                info!(
                    "propagate_gear_ownership: Added LocallyOwned to gear entity {:?}",
                    gear_entity
                );
            }
        }
    }
}

fn cleanup_gear_ownership(
    mut commands: Commands,
    mut removed: RemovedComponents<LocallyOwned>,
    player_q: Query<&PlayerGear>,
    gear_q: Query<Entity, (With<GearMarker>, With<LocallyOwned>)>,
) {
    for player_entity in removed.read() {
        if let Ok(player_gear) = player_q.get(player_entity) {
            let gear_entities = player_gear
                .left_hand
                .into_iter()
                .chain(player_gear.right_hand)
                .chain(player_gear.inventory.iter().copied());

            for gear_entity in gear_entities {
                if gear_q.get(gear_entity).is_ok() {
                    commands.entity(gear_entity).remove::<LocallyOwned>();
                    debug!(
                        "cleanup_gear_ownership: Removed LocallyOwned from gear entity {:?}",
                        gear_entity
                    );
                }
            }
        }
    }
}

/// Client: when PlayerGear changes on a locally-owned entity, update EquipmentPosition
/// for each held/stowed gear item to reflect current slot assignments.
/// This replaces the former ServerSyncPlayerGearMessage forced-overwrite mechanism.
fn reconcile_gear_equipment_positions(
    q_gear: Query<&PlayerGear, (With<LocallyOwned>, Changed<PlayerGear>)>,
    mut commands: Commands,
) {
    for gear in q_gear.iter() {
        if let Some(e) = gear.left_hand {
            commands
                .entity(e)
                .insert(EquipmentPosition::Hand(Hand::Left));
        }
        if let Some(e) = gear.right_hand {
            commands
                .entity(e)
                .insert(EquipmentPosition::Hand(Hand::Right));
        }
        for &e in &gear.inventory {
            commands.entity(e).insert(EquipmentPosition::Stowed);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_client_message::<TruckLoadoutMessage>(Channel::Ordered);
    app.add_mapped_client_message::<RequestGrab>(Channel::Ordered);
    app.add_mapped_client_message::<RequestDrop>(Channel::Ordered);
    app.add_mapped_client_message::<ExportPlayerGearMessage>(Channel::Unreliable);
    app.add_systems(
        Update,
        send_export_player_gear
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
    app.add_systems(
        Update,
        handle_export_player_gear_state
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        Update,
        (handle_request_grab, handle_request_drop).run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        Update,
        (reconcile_gear_on_reconnect, orphan_catcher)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );
    app.add_systems(
        Update,
        hydrate_gear_system
            .run_if(is_pure_client)
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        Update,
        hydrate_player_gear
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        Update,
        handle_truck_loadout_message
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );
    app.add_systems(
        Update,
        handle_truck_loadout_request
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        Update,
        (propagate_gear_ownership, cleanup_gear_ownership)
            .run_if(in_state(UIContextState::InGame))
            .run_if(is_pure_client),
    );
    app.add_systems(
        Update,
        reconcile_gear_equipment_positions
            .run_if(in_state(UIContextState::InGame))
            .run_if(is_pure_client),
    );
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        despawn_gear_on_teardown.run_if(resource_exists::<AuthorityRole>),
    );
}
