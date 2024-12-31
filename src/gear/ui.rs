use super::playergear::{self, Inventory, InventoryNext, InventoryStats, PlayerGear};
use super::{GearSpriteID, GearStuff, GearUsable};
use crate::colors;
use crate::game::GameConfig;
use crate::platform::plt::UI_SCALE;
use crate::player::PlayerSprite;
use crate::root::{GameAssets, GameState};
use bevy::prelude::*;

pub fn keyboard_gear(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    mut gs: GearStuff,
) {
    for (ps, mut playergear) in q_gear.iter_mut() {
        if keyboard_input.just_pressed(ps.controls.cycle) {
            playergear.cycle();
        }
        if keyboard_input.just_pressed(ps.controls.swap) {
            playergear.swap();
        }
        if keyboard_input.just_released(ps.controls.trigger) {
            playergear.right_hand.set_trigger(&mut gs);
        }
        if keyboard_input.just_released(ps.controls.torch) {
            playergear.left_hand.set_trigger(&mut gs);
        }
    }
}

pub fn update_gear_ui(
    gc: Res<GameConfig>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    mut qi: Query<(&Inventory, &mut Sprite), Without<InventoryNext>>,
    mut qs: Query<&mut Text, With<InventoryStats>>,
    mut qin: Query<(&InventoryNext, &mut Sprite), Without<Inventory>>,
) {
    for (ps, playergear) in q_gear.iter() {
        if gc.player_id == ps.id {
            for (inv, mut utai) in qi.iter_mut() {
                let gear = playergear.get_hand(&inv.hand);
                let idx = gear.get_sprite_idx() as usize;

                if utai.texture_atlas.as_ref().unwrap().index != idx {
                    utai.texture_atlas.as_mut().unwrap().index = idx;
                }
            }
            let right_hand_status = playergear.right_hand.get_status();
            for mut txt in qs.iter_mut() {
                if txt.0 != right_hand_status {
                    txt.0.clone_from(&right_hand_status);
                }
            }
            for (inv, mut utai) in qin.iter_mut() {
                // There are 2 possible "None" here, the outside Option::None for when the idx is
                // out of bounds and the inner Gear::None when a slot is empty.
                let next = if let Some(idx) = inv.idx {
                    playergear.get_next(idx).unwrap_or_default()
                } else {
                    playergear.get_next_non_empty().unwrap_or_default()
                };
                let idx = next.get_sprite_idx() as usize;
                if utai.texture_atlas.as_ref().unwrap().index != idx {
                    utai.texture_atlas.as_mut().unwrap().index = idx;
                }
            }
        }
    }
}

pub fn setup_ui_gear_inv_left(p: &mut ChildBuilder, handles: &GameAssets) {
    // Leftmost side panel - inventory
    p.spawn(ImageNode {
        image: handles.images.gear.clone().into(),
        texture_atlas: Some(TextureAtlas {
            index: GearSpriteID::Flashlight2 as usize,
            layout: handles.images.gear_atlas.clone(),
        }),
        ..default()
    })
    .insert(Node {
        width: Val::Px(80.0 * UI_SCALE),
        margin: UiRect::all(Val::Px(-8.0 * UI_SCALE)),
        ..default()
    })
    .insert(playergear::Inventory::new_left());
    p.spawn(Text::new("[TAB]: T.Aux"))
        .insert(TextFont {
            font: handles.fonts.chakra.w300_light.clone(),
            font_size: 16.0 * UI_SCALE,
            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
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
}

pub fn setup_ui_gear_inv_right(p: &mut ChildBuilder, handles: &GameAssets) {
    // Right side panel - inventory
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_grow: 1.0,
        width: Val::Percent(100.0),
        ..default()
    })
    .with_children(|p| {
        p.spawn(ImageNode {
            image: handles.images.gear.clone().into(),
            texture_atlas: Some(TextureAtlas {
                index: GearSpriteID::Flashlight2 as usize,
                layout: handles.images.gear_atlas.clone(),
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
        .insert(playergear::InventoryNext::non_empty());
        p.spawn(ImageNode {
            image: handles.images.gear.clone().into(),
            texture_atlas: Some(TextureAtlas {
                index: GearSpriteID::IonMeter2 as usize,
                layout: handles.images.gear_atlas.clone(),
            }),
            ..default()
        })
        .insert(Node {
            margin: UiRect::left(Val::Px(-8.0)),
            width: Val::Px(80.0 * UI_SCALE),
            ..default()
        })
        .insert(playergear::Inventory::new_right());
        p.spawn(Text::new("-"))
            .insert(TextFont {
                font: handles.fonts.victormono.w600_semibold.clone(),
                font_size: 18.0 * UI_SCALE,
                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
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
                flex_grow: 1.0,
                ..default()
            })
            .insert(TextLayout::default())
            .insert(playergear::InventoryStats);
    });
    p.spawn(Text::new(
        "[Q]: Next  [R]: M.Toggle  [T]: Swap Hands  [C]: Change Evidence",
    ))
    .insert(TextFont {
        font: handles.fonts.chakra.w300_light.clone(),
        font_size: 16.0 * UI_SCALE,
        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
    })
    .insert(TextColor(colors::INVENTORY_STATS_COLOR))
    .insert(Node {
        margin: UiRect::new(
            Val::Px(16.0 * UI_SCALE),
            Val::Px(-8.0 * UI_SCALE),
            Val::Px(-8.0 * UI_SCALE),
            Val::Px(0.0 * UI_SCALE),
        ),
        align_content: AlignContent::End,
        justify_content: JustifyContent::End,
        align_self: AlignSelf::End,
        justify_self: JustifySelf::End,
        ..default()
    })
    .insert(TextLayout::default());
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<GameConfig>()
        .add_systems(FixedUpdate, update_gear_ui)
        .add_systems(Update, keyboard_gear.run_if(in_state(GameState::None)));
}
