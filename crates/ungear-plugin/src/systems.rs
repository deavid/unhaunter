use bevy::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::GameSprite;
use uncommon_states_core::UIContextState;
use ungear_core::assets::GearAssets;
use ungear_core::components::core::GearSprite;
use ungear_core::components::core::StatusText;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::looking_gear::LookingGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::equipment::{Hand, VisualKey};
use ungear_core::types::gear::kind::GearKind;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::PlayerTag;
use unplayer_core::components::{Inventory, InventoryNext, InventoryStats, MainPlayer};
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::resources::sprite_registry::SpriteRegistry;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;

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
    q_main_player: Query<(Entity, Has<PlayerGear>), With<MainPlayer>>,
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
            let player_stats = q_main_player
                .iter()
                .map(|(ent, has_gear)| format!("{:?} (has_gear: {})", ent, has_gear))
                .collect::<Vec<_>>();
            warn!(
                "update_gear_ui: no MainPlayer with PlayerGear found (tick {}). Found players: {:?}",
                *dbg_timer, player_stats
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        update_gear_ui.run_if(in_state(UIContextState::InGame)),
    )
    .add_systems(
        Update,
        update_deployed_gear_sprites.run_if(in_state(UIContextState::InGame)),
    )
    .add_systems(
        Update,
        keyboard_gear.run_if(in_state(UIContextState::InGame)),
    );
}
