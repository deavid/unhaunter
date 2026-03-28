use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use unboard_core::components::mapcolor::MapColor;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::random_seed;
use ungear_core::components::core::GearSprite;
use ungear_core::components::core::StatusText;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::difficulty_ext::DifficultyGearExt;
use ungear_core::events::{
    RequestEquipGearFromVan, RequestUnequipHand, RequestUnequipInventorySlot,
};
use ungear_core::resources::looking_gear::LookingGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::equipment::{Hand, VisualKey};
use ungear_core::types::gear::kind::GearKind;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::{
    Inventory, InventoryNext, InventoryStats, MainPlayer, PlayerSprite,
};
use unrender_std::assets::GearAssets;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::resources::sprite_registry::SpriteRegistry;
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::ownership::{LocallyOwned, Owner, OwnerId};
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untags_core::tags::PlayerTag;
use untruck_core::components::in_truck::InTruck;
use untypes_core::roles::AuthorityRole;
use untypes_core::states::AppState;

use crate::metrics;

fn update_deployed_gear_sprites(
    mut commands: Commands,
    mut q_gear: Query<(Entity, &Position, &GearSprite, Option<&mut Sprite>), With<DeployedGear>>,
    gear_assets: Res<GearAssets>,
    sprite_registry: Res<SpriteRegistry>,
) {
    let measure = metrics::UPDATE_DEPLOYED_GEAR_SPRITES.time_measure();
    for (entity, pos, gear_sprite, sprite) in q_gear.iter_mut() {
        let index = sprite_registry.get(&gear_sprite.0);
        if let Some(mut sprite) = sprite {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = index;
            }
        } else {
            commands.entity(entity).insert((
                Sprite {
                    image: gear_assets.gear.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: gear_assets.gear_layout.clone(),
                        index,
                    }),
                    ..default()
                },
                Transform::from_translation(perspective::to_screen_coord(*pos))
                    .with_scale(Vec3::splat(0.25)),
                Visibility::Inherited,
                GameSprite,
                SpriteLayer::default(),
                MapColor::default(),
            ));
        }
    }
    measure.end_ms();
}

fn keyboard_gear(
    _keyboard_input: Res<ButtonInput<KeyCode>>,
    mut _q_gear: Query<&mut PlayerGear, With<PlayerTag>>,
    _looking_gear: Res<LookingGear>,
    q_in_truck: Query<(), (With<MainPlayer>, With<InTruck>)>,
) {
    if !q_in_truck.is_empty() {
        // TODO: Implement using Entity-based gear
    }
}

fn update_gear_ui(
    q_gear: Query<&PlayerGear, With<MainPlayer>>,
    mut qi: Query<(&Inventory, &mut ImageNode), Without<InventoryNext>>,
    mut qin: Query<(&InventoryNext, &mut ImageNode), Without<Inventory>>,
    mut qs: Query<(&InventoryStats, &mut Text, &mut Node)>,
    q_gearkind: Query<&GearKind>,
    q_status: Query<&StatusText>,
    q_sprite: Query<&GearSprite>,
    gear_registry: Res<GearSpawnerRegistry>,
    sprite_registry: Res<SpriteRegistry>,
    looking_gear: Res<LookingGear>,
    mut dbg_timer: Local<u32>,
) {
    let measure = metrics::UPDATE_GEAR_UI.time_measure();
    let Some(player_gear) = q_gear.iter().next() else {
        *dbg_timer += 1;
        if *dbg_timer % 120 == 1 {
            warn!(
                "update_gear_ui: no MainPlayer with PlayerGear found (tick {})",
                *dbg_timer
            );
        }
        measure.end_ms();
        return;
    };
    *dbg_timer += 1;
    if *dbg_timer % 120 == 1 {
        let left_kind = player_gear.left_hand.map(|e| (e, q_gearkind.get(e).ok()));
        let right_kind = player_gear.right_hand.map(|e| (e, q_gearkind.get(e).ok()));
        debug!(
            "update_gear_ui: left_hand={:?} right_hand={:?} inventory_len={} (tick {})",
            left_kind,
            right_kind,
            player_gear.inventory.len(),
            *dbg_timer
        );
    }

    for (inv, mut image) in qi.iter_mut() {
        let entity = match inv.hand {
            Hand::Left => player_gear.left_hand,
            Hand::Right => player_gear.right_hand,
        };
        let kind = entity
            .and_then(|e| q_gearkind.get(e).ok())
            .unwrap_or(&GearKind::None);

        let sprite_idx = entity
            .and_then(|e| q_sprite.get(e).ok())
            .map(|s| sprite_registry.get(&s.0))
            .or_else(|| {
                gear_registry
                    .metadata
                    .get(kind)
                    .map(|m| sprite_registry.get(&m.sprite_idx))
            })
            .unwrap_or_else(|| sprite_registry.get(&VisualKey::new(VisualKey::NONE)));

        if let Some(atlas) = &mut image.texture_atlas {
            atlas.index = sprite_idx;
        }
    }

    for (inv_next, mut image) in qin.iter_mut() {
        let entity = inv_next.idx.and_then(|idx| player_gear.inventory.get(idx));
        let kind = entity
            .and_then(|e| q_gearkind.get(*e).ok())
            .unwrap_or(&GearKind::None);

        let sprite_idx = entity
            .and_then(|e| q_sprite.get(*e).ok())
            .map(|s| sprite_registry.get(&s.0))
            .or_else(|| {
                gear_registry
                    .metadata
                    .get(kind)
                    .map(|m| sprite_registry.get(&m.sprite_idx))
            })
            .unwrap_or_else(|| sprite_registry.get(&VisualKey::new(VisualKey::NONE)));

        if let Some(atlas) = &mut image.texture_atlas {
            atlas.index = sprite_idx;
        }
    }

    for (stats, mut text, mut node) in qs.iter_mut() {
        let entity = match stats.hand {
            Hand::Left => player_gear.left_hand,
            Hand::Right => player_gear.right_hand,
        };
        let status = entity
            .and_then(|e| q_status.get(e).ok())
            .map(|s| s.0.clone())
            .unwrap_or_default();
        text.0 = status;

        let is_visible = stats.hand == looking_gear.hand() && !text.0.is_empty();
        node.display = if is_visible {
            Display::Flex
        } else {
            Display::None
        };
    }

    measure.end_ms();
}

/// Authority: spawns gear entities and inserts PlayerGear for any player entity that is
/// missing it. Reacts to PlayerSprite entities added by the network layer (setup_mission_players
/// and spawn_late_joining_players). Uses PlayerSprite.network_id and Owner already on the
/// player entity so no LobbyInfo lookup is required.
fn hydrate_player_gear(
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
fn handle_truck_loadout_request(
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        hydrate_player_gear
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(AppState::InGame)),
    );
    app.add_systems(
        Update,
        handle_truck_loadout_request
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(AppState::InGame)),
    );
    app.add_systems(FixedUpdate, update_gear_ui)
        .add_systems(
            Update,
            update_deployed_gear_sprites.run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, keyboard_gear.run_if(in_state(AppState::InGame)));
}
