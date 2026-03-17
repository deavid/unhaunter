use bevy::prelude::*;
use unfoundation_core::colors;
use unfoundation_core::platform::plt::{FONT_SCALE, UI_SCALE};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use unplayer_core::components::{Inventory, InventoryNext, InventoryStats};
use unrender_std::assets::GearAssets;
use unui_core::assets::UiAssets;

pub(crate) fn setup_ui_gear_inv_left(
    p: &mut ChildSpawnerCommands,
    ui_assets: &UiAssets,
    gear_assets: &GearAssets,
) {
    p.spawn(Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        margin: UiRect::left(Val::Px(6.0 * UI_SCALE)),
        ..Default::default()
    })
    .with_children(|p| {
        p.spawn(ImageNode {
            image: gear_assets.gear.clone(),
            texture_atlas: Some(TextureAtlas {
                index: GearSpriteID::Flashlight2 as usize,
                layout: gear_assets.gear_layout.clone(),
            }),
            ..default()
        })
        .insert(Node {
            width: Val::Px(80.0 * UI_SCALE),
            margin: UiRect::all(Val::Px(-8.0 * UI_SCALE)),
            ..default()
        })
        .insert(Inventory::new_left());
        p.spawn(Text::new("[TAB]: T.Aux"))
            .insert(TextFont {
                font: ui_assets.font_chakra_light.clone(),
                font_size: 16.0 * FONT_SCALE,
                ..default()
            })
            .insert(TextColor(colors::INVENTORY_STATS_COLOR))
            .insert(Node {
                margin: UiRect::new(
                    Val::Px(-8.0 * UI_SCALE),
                    Val::Px(-8.0 * UI_SCALE),
                    Val::Px(9.0 * UI_SCALE),
                    Val::Px(-9.0 * UI_SCALE),
                ),
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .insert(TextLayout::default());
    });
    p.spawn(Text::new("-"))
        .insert(TextFont {
            font: ui_assets.font_victor_semibold.clone(),
            font_size: 15.0 * FONT_SCALE,
            ..default()
        })
        .insert(TextColor(colors::INVENTORY_STATS_COLOR))
        .insert(Node {
            justify_content: JustifyContent::Center,
            margin: UiRect::new(
                Val::Px(0.0 * UI_SCALE),
                Val::Px(8.0 * UI_SCALE),
                Val::Px(4.0 * UI_SCALE),
                Val::Px(-16.0 * UI_SCALE),
            ),
            width: Val::Percent(100.0),
            min_width: Val::Px(300.0),
            flex_grow: 1.0,
            ..default()
        })
        .insert(TextLayout::default())
        .insert(InventoryStats::left());
}

pub(crate) fn setup_ui_gear_inv_right(
    p: &mut ChildSpawnerCommands,
    ui_assets: &UiAssets,
    gear_assets: &GearAssets,
) {
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_grow: 1.0,
        width: Val::Percent(100.0),
        ..default()
    })
    .with_children(|p| {
        p.spawn(ImageNode {
            image: gear_assets.gear.clone(),
            texture_atlas: Some(TextureAtlas {
                index: GearSpriteID::Flashlight2 as usize,
                layout: gear_assets.gear_layout.clone(),
            }),
            ..default()
        })
        .insert(Node {
            flex_grow: 0.0,
            flex_shrink: 0.0,
            width: Val::Px(60.0 * UI_SCALE),
            margin: UiRect::new(
                Val::Px(16.0 * UI_SCALE),
                Val::Px(-8.0 * UI_SCALE),
                Val::Px(8.0 * UI_SCALE),
                Val::Px(-8.0 * UI_SCALE),
            ),
            align_self: AlignSelf::Center,
            ..default()
        })
        .insert(InventoryNext::non_empty());
        p.spawn(ImageNode {
            image: gear_assets.gear.clone(),
            texture_atlas: Some(TextureAtlas {
                index: GearSpriteID::IonMeter2 as usize,
                layout: gear_assets.gear_layout.clone(),
            }),
            ..default()
        })
        .insert(Node {
            margin: UiRect::left(Val::Px(-8.0)),
            width: Val::Px(80.0 * UI_SCALE),
            ..default()
        })
        .insert(Inventory::new_right());
        p.spawn(Text::new("-"))
            .insert(TextFont {
                font: ui_assets.font_victor_semibold.clone(),
                font_size: 15.0 * FONT_SCALE,
                ..default()
            })
            .insert(TextColor(colors::INVENTORY_STATS_COLOR))
            .insert(Node {
                justify_content: JustifyContent::Center,
                margin: UiRect::new(
                    Val::Px(0.0 * UI_SCALE),
                    Val::Px(8.0 * UI_SCALE),
                    Val::Px(4.0 * UI_SCALE),
                    Val::Px(-16.0 * UI_SCALE),
                ),
                min_width: Val::Px(300.0),
                flex_grow: 1.0,
                ..default()
            })
            .insert(TextLayout::default())
            .insert(InventoryStats::right());
    });
    p.spawn(Text::new("[R]: M.Toggle"))
        .insert(TextFont {
            font: ui_assets.font_chakra_light.clone(),
            font_size: 16.0 * FONT_SCALE,
            ..default()
        })
        .insert(TextColor(colors::INVENTORY_STATS_COLOR))
        .insert(Node {
            margin: UiRect::new(
                Val::Px(16.0 * UI_SCALE),
                Val::Px(-8.0 * UI_SCALE),
                Val::Px(-8.0 * UI_SCALE),
                Val::Px(0.0 * UI_SCALE),
            ),
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Start,
            ..default()
        })
        .insert(TextLayout::default());
}
