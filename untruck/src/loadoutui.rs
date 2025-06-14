use super::truckgear::TruckGear;
use super::uibutton::{TruckButtonState, TruckButtonType, TruckUIButton};
use bevy::prelude::*;
use uncore::colors;
use uncore::components::game_config::GameConfig;
use uncore::components::player_inventory::{Inventory, InventoryNext};
use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::states::GameState;
use uncore::types::evidence::Evidence;
use uncore::types::evidence_status::EvidenceStatus;
use uncore::types::gear::equipmentposition::Hand;
use uncore::types::gear::spriteid::GearSpriteID;
use uncore::types::gear_kind::GearKind;
use uncore::types::root::game_assets::GameAssets;
use ungear::components::playergear::PlayerGear;
use ungear::gear_usable::GearUsable;
use ungear::types::gear::Gear;
use unstd::materials::UIPanelMaterial;

#[derive(Debug, Component, Clone)]
pub enum LoadoutButton {
    Inventory(Inventory),
    InventoryNext(InventoryNext),
    Van(Gear),
}

#[derive(Debug, Event, Clone)]
pub struct EventButtonClicked(LoadoutButton);

#[derive(Debug, Component, Clone)]
pub struct GearHelp;

#[derive(Debug, Component, Clone)]
pub struct GearHelpTitle;

pub fn setup_loadout_ui(
    p: &mut ChildSpawnerCommands,
    handles: &GameAssets,
    materials: &mut Assets<UIPanelMaterial>,
    difficulty: &CurrentDifficulty,
) {
    let button = || {
        (
            Button,
            BackgroundColor(colors::TRUCKUI_ACCENT2_COLOR),
            BorderColor(colors::TRUCKUI_ACCENT_COLOR),
            Node {
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
        )
    };
    let equipment = |g: GearSpriteID| {
        (
            ImageNode {
                image: handles.images.gear.clone(),
                texture_atlas: Some(TextureAtlas {
                    index: g as usize,
                    layout: handles.images.gear_atlas.clone(),
                }),
                ..default()
            },
            Node {
                width: Val::Px(64.0 * UI_SCALE),
                height: Val::Px(64.0 * UI_SCALE),
                margin: UiRect::all(Val::Px(-4.0)),
                ..default()
            },
        )
    };
    let equipment_def = || equipment(GearSpriteID::None);

    let equipment_frame = |materials: &mut Assets<UIPanelMaterial>| {
        (
            MaterialNode(materials.add(UIPanelMaterial {
                color: colors::TRUCKUI_BGCOLOR.into(),
            })),
            Node {
                padding: UiRect::all(Val::Px(8.0 * UI_SCALE)),
                margin: UiRect::all(Val::Px(2.0 * UI_SCALE)),
                max_height: Val::Px(100.0 * UI_SCALE),
                ..default()
            },
        )
    };
    let left_side = |p: &mut ChildSpawnerCommands| {
        p.spawn((
            Text::new("Player Inventory:"),
            TextFont {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * FONT_SCALE,
                ..default()
            },
            TextColor(colors::TRUCKUI_TEXT_COLOR),
            TextLayout::default(),
            Node {
                margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                ..default()
            },
        ));
        p.spawn(Node {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            flex_grow: 0.04,
            ..default()
        })
        .with_children(|p| {
            p.spawn(equipment_frame(materials)).with_children(|p| {
                p.spawn(button())
                    .insert(LoadoutButton::Inventory(Inventory::new_left()))
                    .with_children(|p| {
                        p.spawn(equipment_def()).insert(Inventory::new_left());
                    });
            });
            p.spawn(equipment_frame(materials)).with_children(|p| {
                p.spawn(button())
                    .insert(LoadoutButton::Inventory(Inventory::new_right()))
                    .with_children(|p| {
                        p.spawn(equipment_def()).insert(Inventory::new_right());
                    });
            });
            p.spawn(equipment_frame(materials)).with_children(|p| {
                for i in 0..2 {
                    p.spawn(button())
                        .insert(LoadoutButton::InventoryNext(InventoryNext::new(i)))
                        .with_children(|p| {
                            p.spawn(equipment_def()).insert(InventoryNext::new(i));
                        });
                }
            });
        });
        p.spawn((
            Text::new("Van Inventory:"),
            TextFont {
                font: handles.fonts.chakra.w300_light.clone(),
                font_size: 25.0 * FONT_SCALE,
                ..default()
            },
            TextColor(colors::TRUCKUI_TEXT_COLOR),
            TextLayout::default(),
            Node {
                margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                ..default()
            },
        ));
        p.spawn(Node { ..default() }).with_children(|p| {
            p.spawn((
                MaterialNode(materials.add(UIPanelMaterial {
                    color: colors::TRUCKUI_BGCOLOR.into(),
                })),
                Node {
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
            ))
            .with_children(|p| {
                let tg = TruckGear::from_difficulty(&difficulty.0);
                for gear in &tg.inventory {
                    p.spawn(button())
                        .insert(LoadoutButton::Van(gear.clone()))
                        .with_children(|p| {
                            p.spawn(equipment(gear.get_sprite_idx()));
                        });
                }
            });
        });
    };
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        ..default()
    })
    .with_children(|p| {
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            flex_basis: Val::Percent(60.0),
            flex_grow: 1.0,
            flex_shrink: 0.0,
            min_width: Val::Px(350.0 * UI_SCALE),
            ..default()
        })
        .with_children(left_side);
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            flex_basis: Val::Percent(40.0),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Text::new("Help and Item description:"),
                TextFont {
                    font: handles.fonts.chakra.w300_light.clone(),
                    font_size: 25.0 * FONT_SCALE,
                    ..default()
                },
                TextColor(colors::TRUCKUI_TEXT_COLOR),
                TextLayout::default(),
                Node {
                    margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    ..default()
                },
                GearHelpTitle,
            ));
            p.spawn((
                Text::new("Select which gear do you want to use to investigate. Click items on the truck inventory to bring them to your inventory. Click on items on your inventory to remove them. Hover items to see the description here."),
                TextFont {
                    font: handles.fonts.titillium.w400_regular.clone(),
                    font_size: 16.0 * FONT_SCALE,
                    ..default()
                },
                TextColor(colors::TRUCKUI_TEXT_COLOR.with_alpha(0.7)),
                TextLayout::default(),
                Node {
                    margin: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    flex_grow: 0.0,
                    flex_shrink: 0.0,
                    flex_basis: Val::Px(400.0 * UI_SCALE),
                    height: Val::Px(220.0 * UI_SCALE),
                    overflow: Overflow::visible(),
                    ..default()
                },
                GearHelp,
            ));
        });
    });
}

fn update_loadout_buttons(
    mut qbut: Query<
        (
            &Interaction,
            &LoadoutButton,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut qh: Query<(&mut Text, Option<&GearHelp>, Option<&GearHelpTitle>)>,
    q_gear: Query<(&PlayerSprite, &PlayerGear)>,
    interaction_query_journal_buttons: Query<&TruckUIButton, With<Button>>,
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
            ev_clk.write(EventButtonClicked(lbut.clone()));
        }
        if *int != Interaction::None {
            // Only update help text if hovered or pressed
            elem = Some(lbut.clone());
        }
    }
    if !changed && elem.is_none() {
        // If nothing changed and nothing is hovered/pressed, don't update help.
        return;
    }

    let Some(p_gear) = q_gear
        .iter()
        .find_map(|(p, g)| if p.id == gc.player_id { Some(g) } else { None })
    else {
        return;
    };

    let gear_for_help = if let Some(lbut) = &elem {
        match lbut {
            LoadoutButton::Inventory(inv) => p_gear.get_hand(&inv.hand),
            LoadoutButton::InventoryNext(invnext) => {
                let idx = invnext.idx.unwrap_or(0); // Default to 0 if None
                p_gear.get_next(idx).unwrap_or_default()
            }
            LoadoutButton::Van(gear) => gear.clone(),
        }
    } else {
        // If nothing is hovered, potentially show help for the currently equipped right-hand item
        // or a generic message. For now, let's use a generic message.
        Gear::none()
    };

    let click_help = if let Some(lbut) = &elem {
        match lbut {
            LoadoutButton::Inventory(inv) => match &inv.hand {
                Hand::Left => "(Click to unequip Left Hand item)",
                Hand::Right => "(Click to unequip Right Hand item)",
            },
            LoadoutButton::InventoryNext(_) => "(Click to unequip Backpack item)",
            LoadoutButton::Van(_) => "(Click to equip item)",
        }
    } else {
        ""
    };

    let (help_title, help_text) = if matches!(gear_for_help.kind, GearKind::None) && elem.is_none()
    {
        // If nothing is hovered AND the "default" gear is None
        (
            "Loadout Management:".to_string(),
            "Select gear from the Van Inventory to add to your Player Inventory. \nClick on items in your Player Inventory to return them to the van. \nHover over any item to see its description and associated evidence here.".to_string(),
        )
    } else {
        let o_evidence = Evidence::try_from(&gear_for_help.kind).ok();
        let ev_state = match o_evidence {
            Some(ev) => interaction_query_journal_buttons // Use the renamed query
                .iter()
                .find(|t| t.class == TruckButtonType::Evidence(ev))
                .map(|t| t.status)
                .unwrap_or(TruckButtonState::Off),
            None => TruckButtonState::Off,
        };
        let status = EvidenceStatus::from_gearkind(o_evidence, ev_state);
        let evidence_text = if status.title.trim().is_empty() {
            "".to_string()
        } else {
            format!(
                "\n\nEvidence: {} ({})",
                status.title.trim().trim_end_matches(':'),
                status.status_desc,
            )
        };
        let gear_name = gear_for_help.get_display_name();
        let gear_desc = gear_for_help.get_description();
        (
            format!("{}:", gear_name),
            format!(
                "{}{}{}{}",
                gear_desc,
                if evidence_text.is_empty() && status.help_text.is_empty() && click_help.is_empty()
                {
                    ""
                } else {
                    "\n"
                },
                evidence_text,
                if status.help_text.is_empty() && click_help.is_empty() {
                    ""
                } else {
                    "\n"
                },
            ),
        )
    };

    for (mut text, ohelp_body, ohelp_title) in &mut qh {
        if ohelp_body.is_some() && help_text != text.0 {
            text.0.clone_from(&help_text);
        }
        if ohelp_title.is_some() && help_title != text.0 {
            text.0.clone_from(&help_title);
        }
    }
}

fn button_clicked(
    mut ev_clk: EventReader<EventButtonClicked>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    gc: Res<GameConfig>,
) {
    let Some(ev) = ev_clk.read().next() else {
        return;
    };
    let Some(mut p_gear) = q_gear
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
            if let Some(idx) = invnext.idx {
                p_gear.take_next(idx);
            }
        }
        LoadoutButton::Van(gear) => {
            p_gear.append(gear.clone());
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (update_loadout_buttons, button_clicked).run_if(in_state(GameState::Truck)),
    );
}
