use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientId, ClientMessageAppExt, FromClient, Replicated};
use uncommon_app_core::random_seed;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::difficulty_ext::DifficultyGearExt;
use ungear_core::events::{
    RequestEquipGearFromVan, RequestUnequipHand, RequestUnequipInventorySlot,
};
use ungear_core::messages::ExportPlayerGearMessage;
use ungear_core::resources::spawner::{GearHydrated, GearMarker, GearSpawnerRegistry};
use ungear_core::types::gear::equipment::Hand;
use ungear_core::types::gear::kind::GearKind;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unreplicon_core::events::PlayerNetworkReconnected;
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole, is_pure_client};

fn from_owner_id(owner_id: OwnerId) -> ClientId {
    match owner_id {
        OwnerId::Server => ClientId::Server,
        OwnerId::Client(e) => ClientId::Client(e),
    }
}

fn send_export_player_gear(
    q_local: Query<&PlayerGear, (With<LocallyOwned>, With<PlayerSprite>)>,
    mut writer: MessageWriter<ExportPlayerGearMessage>,
) {
    for gear in q_local.iter() {
        writer.write(ExportPlayerGearMessage {
            left_hand: gear.left_hand,
            right_hand: gear.right_hand,
            inventory: gear.inventory.clone(),
            held_item: gear.held_item.clone(),
        });
    }
}

fn handle_export_player_gear_state(
    mut reader: MessageReader<FromClient<ExportPlayerGearMessage>>,
    mut q_players: Query<(&Owner, &mut PlayerGear), Without<LocallyOwned>>,
) {
    // TODO: Theoretical race condition: if an unreliable ExportPlayerGearMessage arrives
    // out-of-order AFTER a RequestDrop has been processed, the server might briefly put
    // the dropped item back into the player's inventory.
    for msg in reader.read() {
        for (owner, mut gear) in q_players.iter_mut() {
            if from_owner_id(owner.0) == msg.client_id {
                gear.left_hand = msg.message.left_hand;
                gear.right_hand = msg.message.right_hand;
                gear.inventory = msg.message.inventory.clone();
                gear.held_item = msg.message.held_item.clone();
                break;
            }
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
/// for the local (authority) player. Join-client loadout requests are handled separately by
/// `unreplicon-plugin::handle_truck_loadout_message`.
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

pub(crate) fn app_setup(app: &mut App) {
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
        reconcile_gear_on_reconnect
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
        handle_truck_loadout_request
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        despawn_gear_on_teardown.run_if(resource_exists::<AuthorityRole>),
    );
}
