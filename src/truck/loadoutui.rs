use bevy::prelude::*;

use super::truckgear;
use crate::platform::plt::UI_SCALE;
use crate::{
    colors,
    game::GameConfig,
    gear::{
        self,
        playergear::{self, PlayerGear},
        GearSpriteID, GearUsable,
    },
    materials::UIPanelMaterial,
    player::PlayerSprite,
    root,
};

#[derive(Debug, Component, Clone)]
pub enum LoadoutButton {
    Inventory(playergear::Inventory),
    InventoryNext(playergear::InventoryNext),
    Van(gear::Gear),
}

#[derive(Debug, Event, Clone)]
pub struct EventButtonClicked(LoadoutButton);

pub fn setup_loadout_ui(
    p: &mut ChildBuilder,
    handles: &root::GameAssets,
    materials: &mut Assets<UIPanelMaterial>,
) {
    let button = || ButtonBundle {
        background_color: colors::TRUCKUI_ACCENT2_COLOR.into(),
        border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
        style: Style {
            border: UiRect::all(Val::Px(2.0 * UI_SCALE)),
            margin: UiRect::all(Val::Px(3.0 * UI_SCALE)),
            width: Val::Px(70.0 * UI_SCALE),
            height: Val::Px(74.0 * UI_SCALE),
            ..default()
        },
        ..default()
    };
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
            width: Val::Px(64.0 * UI_SCALE),
            height: Val::Px(64.0 * UI_SCALE),
            ..default()
        },
        ..default()
    };
    let equipment_def = || equipment(GearSpriteID::IonMeterOff);
    let equipment_frame = |materials: &mut Assets<UIPanelMaterial>| MaterialNodeBundle {
        material: materials.add(UIPanelMaterial {
            color: colors::TRUCKUI_BGCOLOR,
        }),

        style: Style {
            padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
            margin: UiRect::all(Val::Px(2.0 * UI_SCALE)),
            ..default()
        },
        ..default()
    };
    p.spawn(
        TextBundle::from_section(
            "Player Inventory:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * UI_SCALE,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
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
        p.spawn(equipment_frame(materials)).with_children(|p| {
            p.spawn(button())
                .insert(LoadoutButton::Inventory(playergear::Inventory::new_left()))
                .with_children(|p| {
                    p.spawn(equipment_def())
                        .insert(playergear::Inventory::new_left());
                });
        });
        p.spawn(equipment_frame(materials)).with_children(|p| {
            p.spawn(button())
                .insert(LoadoutButton::Inventory(playergear::Inventory::new_right()))
                .with_children(|p| {
                    p.spawn(equipment_def())
                        .insert(playergear::Inventory::new_right());
                });
        });
        p.spawn(equipment_frame(materials)).with_children(|p| {
            for i in 0..2 {
                p.spawn(button())
                    .insert(LoadoutButton::InventoryNext(
                        playergear::InventoryNext::new(i),
                    ))
                    .with_children(|p| {
                        p.spawn(equipment_def())
                            .insert(playergear::InventoryNext::new(i));
                    });
            }
        });
    });
    p.spawn(
        TextBundle::from_section(
            "Van Inventory:",
            TextStyle {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * UI_SCALE,
                color: colors::TRUCKUI_TEXT_COLOR,
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
            ..default()
        }),
    );
    p.spawn(NodeBundle {
        style: Style { ..default() },
        ..default()
    })
    .with_children(|p| {
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
                row_gap: Val::Px(6.0 * UI_SCALE),
                column_gap: Val::Px(6.0 * UI_SCALE),
                min_height: Val::Px(200.0 * UI_SCALE),
                max_width: Val::Px(600.0 * UI_SCALE),
                padding: UiRect::all(Val::Px(12.0 * UI_SCALE)),
                margin: UiRect::all(Val::Px(2.0 * UI_SCALE)),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            let tg = truckgear::TruckGear::new();
            for gear in &tg.inventory {
                p.spawn(button())
                    .insert(LoadoutButton::Van(gear.clone()))
                    .with_children(|p| {
                        p.spawn(equipment(gear.get_sprite_idx()));
                    });
            }
        });
    });
}

pub fn update_loadout_buttons(
    mut qbut: Query<
        (
            &Interaction,
            &LoadoutButton,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut ev_clk: EventWriter<EventButtonClicked>,
) {
    for (int, lbut, mut border, mut bg) in &mut qbut {
        let bgalpha = match int {
            Interaction::Pressed => 1.0,
            Interaction::Hovered => 0.2,
            Interaction::None => 0.01,
        };
        let bdalpha = match int {
            Interaction::Pressed => 1.0,
            Interaction::Hovered => 0.5,
            Interaction::None => 0.01,
        };
        border.0 = colors::TRUCKUI_ACCENT_COLOR.with_a(bdalpha);
        bg.0 = colors::TRUCKUI_ACCENT2_COLOR.with_a(bgalpha);
        if *int == Interaction::Pressed {
            ev_clk.send(EventButtonClicked(lbut.clone()));
        }
    }
}

pub fn button_clicked(
    mut ev_clk: EventReader<EventButtonClicked>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    gc: Res<GameConfig>,
) {
    let Some(ev) = ev_clk.read().next() else {
        return;
    };
    let Some(mut p_gear) =
        q_gear
            .iter_mut()
            .find_map(|(p, g)| if p.id == gc.player_id { Some(g) } else { None })
    else {
        return;
    };

    match &ev.0 {
        LoadoutButton::Inventory(inv) => {
            p_gear.take_hand(&inv.hand);
        }
        LoadoutButton::InventoryNext(invnext) => {
            p_gear.take_next(invnext.idx.expect("Truck UI should always specify IDX"));
        }
        LoadoutButton::Van(gear) => {
            p_gear.append(gear.clone());
        }
    }
}
