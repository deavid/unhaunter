use bevy::prelude::*;

use crate::{
    colors,
    gear::{playergear, GearSpriteID, GearUsable},
    materials::UIPanelMaterial,
    root,
};

use super::truckgear;

pub fn setup_loadout_ui(
    p: &mut ChildBuilder,
    handles: &root::GameAssets,
    materials: &mut Assets<UIPanelMaterial>,
) {
    let equipment = |g: GearSpriteID| AtlasImageBundle {
        image: UiImage {
            texture: handles.images.gear.clone(),
            flip_x: false,
            flip_y: false,
        },
        texture_atlas: TextureAtlas {
            index: g as usize,
            layout: handles.images.gear_atlas.clone(),
        },
        style: Style {
            width: Val::Px(64.0),
            height: Val::Px(64.0),
            ..default()
        },
        ..default()
    };
    let equipment_def = || equipment(GearSpriteID::IonMeterOff);
    let gear_button = |materials: &mut Assets<UIPanelMaterial>| MaterialNodeBundle {
        material: materials.add(UIPanelMaterial {
            color: colors::TRUCKUI_BGCOLOR,
        }),

        style: Style {
            padding: UiRect::all(Val::Px(4.0)),
            margin: UiRect::all(Val::Px(2.0)),
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        ..default()
    };
    p.spawn(
        TextBundle::from_section(
            "Player Inventory:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0)),
            ..default()
        }),
    );
    p.spawn(NodeBundle {
        style: Style {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            flex_grow: 0.04,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        ..default()
    })
    .with_children(|p| {
        p.spawn(gear_button(materials)).with_children(|p| {
            p.spawn(equipment_def())
                .insert(playergear::Inventory::new_left());
        });
        p.spawn(gear_button(materials)).with_children(|p| {
            p.spawn(equipment_def())
                .insert(playergear::Inventory::new_right());
        });
        p.spawn(gear_button(materials)).with_children(|p| {
            for i in 0..4 {
                p.spawn(equipment_def())
                    .insert(playergear::InventoryNext::new(i));
            }
        });
    });
    p.spawn(
        TextBundle::from_section(
            "Van Inventory:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0)),
            ..default()
        }),
    );
    p.spawn(MaterialNodeBundle {
        material: materials.add(UIPanelMaterial {
            color: colors::TRUCKUI_BGCOLOR,
        }),
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(6, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(5, 1.0),
            grid_auto_flow: GridAutoFlow::Row,
            flex_grow: 0.7,
            min_height: Val::Px(200.0),
            max_width: Val::Px(600.0),
            margin: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        ..default()
    })
    .with_children(|p| {
        let tg = truckgear::TruckGear::new();
        for gear in &tg.inventory {
            p.spawn(equipment(gear.get_sprite_idx()));
        }
    });
}
