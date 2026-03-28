use crate::evidence_status::EvidenceStatus;
use bevy::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::colors;
use unfoundation_core::platform::plt::{FONT_SCALE, UI_SCALE};
use ungear_core::components::playergear::PlayerGear;
use ungear_core::difficulty_ext::DifficultyGearExt;
use ungear_core::events::{
    RequestEquipGearFromVan, RequestUnequipHand, RequestUnequipInventorySlot,
};
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::gear::equipment::{Hand, VisualKey};
use ungear_core::types::gear::kind::GearKind;
use unghost_core::types::evidence::Evidence;
use unplayer_core::components::{Inventory, InventoryNext};
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unrender_std::assets::GearAssets;
use unrender_std::materials::UIPanelMaterial;
use unrender_std::resources::sprite_registry::SpriteRegistry;
use unreplicon_core::messages::{TruckLoadoutAction, TruckLoadoutMessage};
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};
use untypes_core::states::GameState;
use unui_core::assets::UiAssets;

#[derive(Debug, Component, Clone)]
pub(crate) enum LoadoutButton {
    Inventory(Inventory),
    InventoryNext(InventoryNext),
    Van(GearKind),
}

#[derive(Debug, Message, Clone)]
pub(crate) struct EventButtonClicked(pub(crate) LoadoutButton);

#[derive(Debug, Component, Clone)]
pub(crate) struct GearHelp;

#[derive(Debug, Component, Clone)]
pub(crate) struct GearHelpTitle;

pub(crate) fn setup_loadout_ui(
    p: &mut ChildSpawnerCommands,
    ui_assets: &UiAssets,
    gear_assets: &GearAssets,
    materials: &mut Assets<UIPanelMaterial>,
    difficulty: &CurrentDifficulty,
    gear_registry: &GearSpawnerRegistry,
    sprite_registry: &SpriteRegistry,
) {
    let button = || {
        (
            Button,
            BackgroundColor(colors::TRUCKUI_ACCENT2_COLOR),
            BorderColor::all(colors::TRUCKUI_ACCENT_COLOR),
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
    let equipment = |g: VisualKey| {
        (
            ImageNode {
                image: gear_assets.gear.clone(),
                texture_atlas: Some(TextureAtlas {
                    index: sprite_registry.get(&g),
                    layout: gear_assets.gear_layout.clone(),
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
    let equipment_def = || equipment(VisualKey::new(VisualKey::NONE));

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
                font: ui_assets.font_chakra_light.clone(),
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
                font: ui_assets.font_chakra_light.clone(),
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
                for gear_kind in &difficulty.0.truck_gear() {
                    let sprite_idx = gear_registry
                        .metadata
                        .get(gear_kind)
                        .map(|m| m.sprite_idx.clone())
                        .unwrap_or_else(|| VisualKey::new(VisualKey::NONE));
                    p.spawn(button())
                        .insert(LoadoutButton::Van(*gear_kind))
                        .with_children(|p| {
                            p.spawn(equipment(sprite_idx));
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
                    font: ui_assets.font_chakra_light.clone(),
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
                    font: ui_assets.font_titillium_regular.clone(),
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
    q_gear: Query<(&PlayerSprite, &PlayerGear, Has<MainPlayer>)>,
    q_gearkind: Query<&GearKind>,
    interaction_query_journal_buttons: Query<&TruckUIButton, With<Button>>,
    mut ev_clk: MessageWriter<EventButtonClicked>,
    gear_registry: Res<GearSpawnerRegistry>,
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
        *border = BorderColor::all(colors::TRUCKUI_ACCENT_COLOR.with_alpha(bdalpha));
        *bg = BackgroundColor(colors::TRUCKUI_ACCENT2_COLOR.with_alpha(bgalpha));
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

    let Some(_p_gear) = q_gear
        .iter()
        .find_map(|(_p, g, is_main)| if is_main { Some(g) } else { None })
    else {
        return;
    };

    let gear_kind_for_help = if let Some(lbut) = &elem {
        match lbut {
            LoadoutButton::Inventory(inv) => {
                let entity = match inv.hand {
                    Hand::Left => _p_gear.left_hand,
                    Hand::Right => _p_gear.right_hand,
                };
                entity
                    .and_then(|e| q_gearkind.get(e).ok())
                    .cloned()
                    .unwrap_or(GearKind::None)
            }
            LoadoutButton::InventoryNext(invnext) => invnext
                .idx
                .and_then(|idx| _p_gear.inventory.get(idx))
                .and_then(|e| q_gearkind.get(*e).ok())
                .cloned()
                .unwrap_or(GearKind::None),
            LoadoutButton::Van(kind) => *kind,
        }
    } else {
        GearKind::None
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

    let (help_title, help_text) = if matches!(gear_kind_for_help, GearKind::None) && elem.is_none()
    {
        (
            "Loadout Management:".to_string(),
            "Select gear from the Van Inventory to add to your Player Inventory. \nClick on items in your Player Inventory to return them to the van. \nHover over any item to see its description and associated evidence here.".to_string(),
        )
    } else {
        let o_evidence = Evidence::try_from(&gear_kind_for_help).ok();
        let ev_state = match o_evidence {
            Some(ev) => interaction_query_journal_buttons
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
        let (gear_name, gear_desc) = gear_registry
            .metadata
            .get(&gear_kind_for_help)
            .map(|m| (m.name.as_str(), m.description.as_str()))
            .unwrap_or(("None", ""));

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

fn update_loadout_icons(
    q_gear: Query<(&PlayerSprite, &PlayerGear, Has<MainPlayer>)>,
    q_gearkind: Query<&GearKind>,
    q_but: Query<(&LoadoutButton, &Children)>,
    mut q_image: Query<&mut ImageNode>,
    gear_registry: Res<GearSpawnerRegistry>,
    sprite_registry: Res<SpriteRegistry>,
) {
    let Some(p_gear) = q_gear
        .iter()
        .find_map(|(_p, g, is_main)| if is_main { Some(g) } else { None })
    else {
        return;
    };

    for (lbut, children) in q_but.iter() {
        let kind = match lbut {
            LoadoutButton::Inventory(inv) => {
                let entity = match inv.hand {
                    Hand::Left => p_gear.left_hand,
                    Hand::Right => p_gear.right_hand,
                };
                entity
                    .and_then(|e| q_gearkind.get(e).ok())
                    .cloned()
                    .unwrap_or(GearKind::None)
            }
            LoadoutButton::InventoryNext(invnext) => invnext
                .idx
                .and_then(|idx| p_gear.inventory.get(idx))
                .and_then(|e| q_gearkind.get(*e).ok())
                .cloned()
                .unwrap_or(GearKind::None),
            LoadoutButton::Van(kind) => *kind,
        };

        for &child in children {
            if let Ok(mut image) = q_image.get_mut(child)
                && let Some(atlas) = &mut image.texture_atlas
            {
                let sprite_idx = gear_registry
                    .metadata
                    .get(&kind)
                    .map(|m| sprite_registry.get(&m.sprite_idx))
                    .unwrap_or_else(|| sprite_registry.get(&VisualKey::new(VisualKey::NONE)));
                atlas.index = sprite_idx;
            }
        }
    }
}

fn button_clicked(
    mut ev_clk: MessageReader<EventButtonClicked>,
    q_gear: Query<(&PlayerSprite, &PlayerGear, Has<MainPlayer>)>,
    authority: Option<Res<untypes_core::roles::AuthorityRole>>,
    mut ev_loadout: MessageWriter<TruckLoadoutMessage>,
    mut ev_equip_van: MessageWriter<RequestEquipGearFromVan>,
    mut ev_unequip_hand: MessageWriter<RequestUnequipHand>,
    mut ev_unequip_slot: MessageWriter<RequestUnequipInventorySlot>,
) {
    let Some(ev) = ev_clk.read().next() else {
        return;
    };

    let Some(p_gear) = q_gear
        .iter()
        .find_map(|(_p, g, is_main)| if is_main { Some(g) } else { None })
    else {
        warn!(
            "button_clicked: no MainPlayer entity found; ignoring {:?}",
            ev.0
        );
        return;
    };
    match &ev.0 {
        LoadoutButton::Inventory(inv) => {
            let entity = match inv.hand {
                Hand::Left => p_gear.left_hand,
                Hand::Right => p_gear.right_hand,
            };
            if entity.is_none() {
                warn!(
                    "Inventory button clicked but no item in hand {:?}!",
                    inv.hand
                );
                return;
            }
            if authority.is_none() {
                // Pure Client: Send the intent to the server.
                ev_loadout.write(TruckLoadoutMessage {
                    action: TruckLoadoutAction::ClearHand(inv.hand),
                });
                return;
            }

            // Authority (Host): emit the domain event.
            ev_unequip_hand.write(RequestUnequipHand { hand: inv.hand });
        }
        LoadoutButton::InventoryNext(invnext) => {
            let Some(idx) = invnext.idx else {
                warn!("InventoryNext button clicked but no index!");
                return;
            };
            if idx >= p_gear.inventory.len() {
                warn!(
                    "InventoryNext button clicked but index {} is out of bounds!",
                    idx
                );
                return;
            }

            if authority.is_none() {
                // Pure Client: Send the intent to the server.
                ev_loadout.write(TruckLoadoutMessage {
                    action: TruckLoadoutAction::ClearInventorySlot(idx),
                });
                return;
            }

            // Authority (Host): emit the domain event.
            ev_unequip_slot.write(RequestUnequipInventorySlot { idx });
        }
        LoadoutButton::Van(kind) => {
            if *kind == GearKind::None {
                warn!("Van button clicked but GearKind::None!");
                return;
            }

            if authority.is_none() {
                // Pure Client: Send the intent to the server.
                ev_loadout.write(TruckLoadoutMessage {
                    action: TruckLoadoutAction::AddGear(*kind),
                });
                return;
            }

            // Authority (Host): emit the domain event.
            ev_equip_van.write(RequestEquipGearFromVan { kind: *kind });
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (update_loadout_buttons, update_loadout_icons, button_clicked)
            .run_if(in_state(GameState::Truck)),
    );
}
