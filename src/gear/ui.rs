use super::playergear::{self, Inventory, InventoryNext, InventoryStats, PlayerGear};
use super::{GearSpriteID, GearStuff, GearUsable};
use crate::colors;
use crate::game::GameConfig;
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
    mut qi: Query<(&Inventory, &mut TextureAtlas), Without<InventoryNext>>,
    mut qs: Query<&mut Text, With<InventoryStats>>,
    mut qin: Query<(&InventoryNext, &mut TextureAtlas), Without<Inventory>>,
) {
    for (ps, playergear) in q_gear.iter() {
        if gc.player_id == ps.id {
            for (inv, mut utai) in qi.iter_mut() {
                let gear = playergear.get_hand(&inv.hand);
                let idx = gear.get_sprite_idx() as usize;
                if utai.index != idx {
                    utai.index = idx;
                }
            }
            let right_hand_status = playergear.right_hand.get_status();
            for mut txt in qs.iter_mut() {
                if txt.sections[0].value != right_hand_status {
                    txt.sections[0].value = right_hand_status.clone();
                }
            }
            for (inv, mut utai) in qin.iter_mut() {
                // There are 2 possible "None" here, the outside Option::None for
                // when the idx is out of bounds and the inner Gear::None when a
                // slot is empty.
                let next = playergear.get_next(inv.idx).unwrap_or_default();
                let idx = next.get_sprite_idx() as usize;
                if utai.index != idx {
                    utai.index = idx;
                }
            }
        }
    }
}

pub fn setup_ui_gear_inv_left(parent: &mut ChildBuilder, handles: &GameAssets) {
    // Leftmost side panel - inventory
    parent
        .spawn(AtlasImageBundle {
            image: UiImage {
                texture: handles.images.gear.clone(),
                flip_x: false,
                flip_y: false,
            },
            texture_atlas: TextureAtlas {
                index: GearSpriteID::Flashlight2 as usize,
                layout: handles.images.gear_atlas.clone(),
            },
            ..default()
        })
        .insert(playergear::Inventory::new_left());
}

pub fn setup_ui_gear_inv_right(parent: &mut ChildBuilder, handles: &GameAssets) {
    // Right side panel - inventory
    parent
        .spawn(AtlasImageBundle {
            image: UiImage {
                texture: handles.images.gear.clone(),
                flip_x: false,
                flip_y: false,
            },
            texture_atlas: TextureAtlas {
                index: GearSpriteID::Flashlight2 as usize,
                layout: handles.images.gear_atlas.clone(),
            },
            background_color: Color::GRAY.with_l(0.8).with_a(0.8).into(),
            style: Style {
                flex_grow: 0.0,
                flex_shrink: 0.0,
                width: Val::Px(60.0),
                margin: UiRect::new(Val::Px(16.0), Val::Px(-8.0), Val::Px(8.0), Val::Px(4.0)),
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        })
        .insert(playergear::InventoryNext::new(0));

    parent
        .spawn(AtlasImageBundle {
            image: UiImage {
                texture: handles.images.gear.clone(),
                flip_x: false,
                flip_y: false,
            },
            texture_atlas: TextureAtlas {
                index: GearSpriteID::IonMeter2 as usize,
                layout: handles.images.gear_atlas.clone(),
            },
            style: Style {
                margin: UiRect::left(Val::Px(-8.0)),
                ..default()
            },
            ..default()
        })
        .insert(playergear::Inventory::new_right());
    let mut text_bundle = TextBundle::from_section(
        "-",
        TextStyle {
            font: handles.fonts.victormono.w600_semibold.clone(),
            font_size: 20.0,
            color: colors::INVENTORY_STATS_COLOR,
        },
    );
    text_bundle.style = Style {
        flex_grow: 1.0,
        ..default()
    };
    parent.spawn(text_bundle).insert(playergear::InventoryStats);
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<GameConfig>()
        .add_systems(FixedUpdate, update_gear_ui)
        .add_systems(Update, keyboard_gear.run_if(in_state(GameState::None)));
}
