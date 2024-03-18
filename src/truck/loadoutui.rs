use bevy::prelude::*;

use crate::{
    colors,
    gear::{playergear, GearSpriteID, GearUsable},
    root,
};

use super::truckgear;

pub fn setup_loadout_ui(p: &mut ChildBuilder, handles: &root::GameAssets) {
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
            flex_grow: 0.4,
            flex_wrap: FlexWrap::Wrap,
            min_height: Val::Px(100.0),
            ..default()
        },
        ..default()
    })
    .with_children(|p| {
        p.spawn(equipment_def())
            .insert(playergear::Inventory::new_left());
        p.spawn(equipment_def())
            .insert(playergear::Inventory::new_right());

        for i in 0..8 {
            p.spawn(equipment_def())
                .insert(playergear::InventoryNext::new(i));
        }
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
    p.spawn(NodeBundle {
        background_color: colors::TRUCKUI_BGCOLOR.into(),
        style: Style {
            justify_content: JustifyContent::FlexStart,
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(8, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
            grid_auto_flow: GridAutoFlow::Row,
            flex_grow: 0.7,
            min_height: Val::Px(200.0),
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
