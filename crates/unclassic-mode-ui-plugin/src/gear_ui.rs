use bevy::picking::events::{Click, Pointer};
use bevy::prelude::*;
use unclassic_mode_core::colors;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uncommon_states_core::UIContextState;
use ungear_core::assets::GearAssets;
use ungear_core::types::gear::sprite_id::GearSpriteID;
use uninput_core::components::PlayerInput;
use uninput_core::resources::MissionInputFocus;
use unplayer_core::components::{Inventory, InventoryNext, InventoryStats};
use unplayer_core::components::{MainPlayer, PlayerSprite};

use crate::assets::GameUiAssets;

#[derive(Component, Debug)]
struct LeftHandGearClickTarget;

#[derive(Component, Debug)]
struct RightHandGearClickTarget;

#[derive(Component, Debug)]
struct NextGearClickTarget;

#[derive(Component, Debug)]
struct SwapHandsClickTarget;

fn gear_ui_click_system(
    mut click_events: MessageReader<Pointer<Click>>,
    q_left: Query<(), With<LeftHandGearClickTarget>>,
    q_right: Query<(), With<RightHandGearClickTarget>>,
    q_next: Query<(), With<NextGearClickTarget>>,
    q_swap: Query<(), With<SwapHandsClickTarget>>,
    mut q_player: Query<&mut PlayerInput, (With<PlayerSprite>, With<MainPlayer>)>,
    focus: Res<MissionInputFocus>,
) {
    if !focus.has_focus {
        return;
    }

    for click_event in click_events.read() {
        if click_event.button != PointerButton::Primary {
            continue;
        }

        let Ok(mut player_input) = q_player.single_mut() else {
            return;
        };

        if q_left.contains(click_event.entity) {
            player_input.use_left_hand = true;
        } else if q_right.contains(click_event.entity) {
            player_input.use_right_hand = true;
        } else if q_next.contains(click_event.entity) {
            player_input.inventory_cycle = true;
        } else if q_swap.contains(click_event.entity) {
            player_input.inventory_swap = true;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        gear_ui_click_system
            .in_set(uninput_core::PlayerInputSet)
            .run_if(in_state(UIContextState::InGame)),
    );
}

pub(crate) fn setup_ui_gear_inv_left(
    p: &mut ChildSpawnerCommands,
    ui_assets: &GameUiAssets,
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
        .insert(Inventory::new_left())
        .insert(LeftHandGearClickTarget)
        .insert(Pickable::default());
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
    ui_assets: &GameUiAssets,
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
        .insert(InventoryNext::non_empty())
        .insert(NextGearClickTarget)
        .insert(Pickable::default());
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
        .insert(Inventory::new_right())
        .insert(RightHandGearClickTarget)
        .insert(Pickable::default());
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
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_grow: 1.0,
        width: Val::Percent(100.0),
        ..default()
    })
    .with_children(|p| {
        p.spawn(Text::new("[T]: Swap Hands"))
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
                    Val::Px(0.0 * UI_SCALE),
                    Val::Px(-2.0 * UI_SCALE),
                ),
                align_content: AlignContent::Start,
                justify_content: JustifyContent::Start,
                align_self: AlignSelf::Start,
                justify_self: JustifySelf::Start,
                ..default()
            })
            .insert(TextLayout::default())
            .insert(SwapHandsClickTarget)
            .insert(Pickable::default());
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
                    Val::Px(0.0 * UI_SCALE),
                    Val::Px(-2.0 * UI_SCALE),
                ),
                align_content: AlignContent::Start,
                justify_content: JustifyContent::Start,
                align_self: AlignSelf::Start,
                justify_self: JustifySelf::Start,
                ..default()
            })
            .insert(TextLayout::default());
    });
}
