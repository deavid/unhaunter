use super::truckgear;
use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use crate::difficulty::CurrentDifficulty;
use crate::game::evidence::EvidenceStatus;
use crate::gear::{Gear, GearKind};
use crate::ghost_definitions::Evidence;
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
use bevy::prelude::*;

#[derive(Debug, Component, Clone)]
pub enum LoadoutButton {
    Inventory(playergear::Inventory),
    InventoryNext(playergear::InventoryNext),
    Van(gear::Gear),
}

#[derive(Debug, Event, Clone)]
pub struct EventButtonClicked(LoadoutButton);
#[derive(Debug, Component, Clone)]
pub struct GearHelp;

pub fn setup_loadout_ui(
    p: &mut ChildBuilder,
    handles: &root::GameAssets,
    materials: &mut Assets<UIPanelMaterial>,
    difficulty: &CurrentDifficulty,
) {
    let button = || ButtonBundle {
        background_color: colors::TRUCKUI_ACCENT2_COLOR.into(),
        border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
        style: Style {
            justify_content: JustifyContent::Center,
            justify_items: JustifyItems::Center,
            justify_self: JustifySelf::Center,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            border: UiRect::all(Val::Px(2.0 * UI_SCALE)),
            margin: UiRect::all(Val::Px(3.0 * UI_SCALE)),
            max_width: Val::Px(70.0 * UI_SCALE),
            max_height: Val::Px(74.0 * UI_SCALE),
            ..default()
        },
        ..default()
    };
    let equipment = || ImageBundle {
        image: UiImage {
            texture: handles.images.gear.clone(),
            flip_x: false,
            flip_y: false,
            ..default()
        },
        style: Style {
            width: Val::Px(64.0 * UI_SCALE),
            height: Val::Px(64.0 * UI_SCALE),
            margin: UiRect::all(Val::Px(-4.0)),
            ..default()
        },
        ..default()
    };
    let equipment_atlas = |g: GearSpriteID| TextureAtlas {
        index: g as usize,
        layout: handles.images.gear_atlas.clone(),
    };
    let equipment_atlas_def = || equipment_atlas(GearSpriteID::IonMeterOff);
    let equipment_frame = |materials: &mut Assets<UIPanelMaterial>| MaterialNodeBundle {
        material: materials.add(UIPanelMaterial {
            color: colors::TRUCKUI_BGCOLOR.into(),
        }),
        style: Style {
            padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
            margin: UiRect::all(Val::Px(2.0 * UI_SCALE)),
            max_height: Val::Px(100.0 * UI_SCALE),
            ..default()
        },
        ..default()
    };
    let left_side = |p: &mut ChildBuilder| {
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
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            p.spawn(equipment_frame(materials)).with_children(|p| {
                p.spawn(button())
                    .insert(LoadoutButton::Inventory(playergear::Inventory::new_left()))
                    .with_children(|p| {
                        p.spawn(equipment())
                            .insert(equipment_atlas_def())
                            .insert(playergear::Inventory::new_left());
                    });
            });
            p.spawn(equipment_frame(materials)).with_children(|p| {
                p.spawn(button())
                    .insert(LoadoutButton::Inventory(playergear::Inventory::new_right()))
                    .with_children(|p| {
                        p.spawn(equipment())
                            .insert(equipment_atlas_def())
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
                            p.spawn(equipment())
                                .insert(equipment_atlas_def())
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
                    color: colors::TRUCKUI_BGCOLOR.into(),
                }),
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(5, 1.0),
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
                let tg = truckgear::TruckGear::from_difficulty(&difficulty.0);
                for gear in &tg.inventory {
                    p.spawn(button())
                        .insert(LoadoutButton::Van(gear.clone()))
                        .with_children(|p| {
                            p.spawn(equipment())
                                .insert(equipment_atlas(gear.get_sprite_idx()));
                        });
                }
            });
        });
    };
    p.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        ..default()
    })
    .with_children(|p| {
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                flex_basis: Val::Percent(60.0),
                flex_grow: 1.0,
                flex_shrink: 0.0,
                min_width: Val::Px(350.0 * UI_SCALE),
                ..default()
            },
            ..default()
        })
        .with_children(left_side);
        p.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                flex_basis: Val::Percent(40.0),
                flex_grow: 0.0,
                flex_shrink: 1.0,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            // ---- Right side of the window ----
            p.spawn(
                TextBundle::from_section(
                    "Help and Item description:",
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
            p.spawn(
                TextBundle::from_section(
                    "Select ...",
                    TextStyle {
                        font: handles.fonts.titillium.w400_regular.clone(),
                        font_size: 22.0 * UI_SCALE,
                        color: colors::TRUCKUI_TEXT_COLOR.with_alpha(0.7),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    flex_grow: 0.0,
                    flex_shrink: 0.0,
                    flex_basis: Val::Px(400.0 * UI_SCALE),
                    height: Val::Px(220.0 * UI_SCALE),
                    overflow: Overflow::visible(),
                    ..default()
                }),
            )
            .insert(GearHelp);
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
    mut qh: Query<&mut Text, With<GearHelp>>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    interaction_query: Query<&TruckUIButton, With<Button>>,
    mut ev_clk: EventWriter<EventButtonClicked>,
    gc: Res<GameConfig>,
) {
    let mut changed = false;
    let mut elem = None;
    for (int, lbut, mut border, mut bg) in &mut qbut {
        changed = true;
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
        border.0 = colors::TRUCKUI_ACCENT_COLOR.with_alpha(bdalpha);
        bg.0 = colors::TRUCKUI_ACCENT2_COLOR.with_alpha(bgalpha);
        if *int == Interaction::Pressed {
            ev_clk.send(EventButtonClicked(lbut.clone()));
        }
        if *int != Interaction::None {
            elem = Some(lbut.clone());
        }
    }
    if !changed {
        return;
    }
    let Some(p_gear) = q_gear
        .iter()
        .find_map(|(p, g)| if p.id == gc.player_id { Some(g) } else { None })
    else {
        return;
    };
    let gear = if let Some(lbut) = &elem {
        match lbut {
            LoadoutButton::Inventory(inv) => p_gear.get_hand(&inv.hand),
            LoadoutButton::InventoryNext(invnext) => {
                let idx = invnext.idx.unwrap_or_default();
                p_gear.get_next(idx).unwrap_or_default()
            }
            LoadoutButton::Van(gear) => gear.clone(),
        }
    } else {
        Gear::none()
    };
    let click_help = if let Some(lbut) = &elem {
        match lbut {
            LoadoutButton::Inventory(inv) => match &inv.hand {
                playergear::Hand::Left => "(Click to remove the item from your Left Hand)",
                playergear::Hand::Right => "(Click to remove the item from your Right Hand)",
            },
            LoadoutButton::InventoryNext(_) => "(Click to remove the item from your Backpack)",
            LoadoutButton::Van(_) => "(Click to add the item to your Inventory's first empty slot)",
        }
    } else {
        ""
    };
    let help_text = if matches!(gear.kind, GearKind::None) {
        "Select which gear do you want to use to investigate. Click items on the truck inventory to bring them to your inventory. Click on items on your inventory to remove them. Hover items to see the description here.".to_string()
    } else {
        let o_evidence = Evidence::try_from(&gear.kind).ok();
        let ev_state = match o_evidence {
            Some(ev) => interaction_query
                .iter()
                .find(|t| t.class == TruckButtonType::Evidence(ev))
                .map(|t| t.status)
                .unwrap_or(TruckButtonState::Off),
            None => TruckButtonState::Off,
        };
        let status = EvidenceStatus::from_gearkind(o_evidence, ev_state);
        let gear_name = gear.get_display_name();
        let gear_desc = gear.get_description();
        format!(
            "{gear_name}: {gear_desc}\n{}{}{}\n{click_help}",
            status.title, status.status, status.help_text
        )
    };
    for mut text in &mut qh {
        if help_text != text.sections[0].value {
            text.sections[0].value.clone_from(&help_text);
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
