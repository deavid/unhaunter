use super::{activity, journalui, loadoutui, sanity, sensors};
use crate::assets::TruckUiAssets;
use crate::colors;
use bevy::prelude::*;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uncommon_app_core::states::AppState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use uninput_core::states::InGameUiState;
use unrender_std::assets::GearAssets;
use unrender_std::custom_material2::UIPanelMaterial;
use unrender_std::resources::sprite_registry::SpriteRegistry;
use untruck_core::components::truck_tab::TruckTab;
use untruck_core::components::truck_ui_button::TruckButtonTypeExt;
use untruck_core::components::truck_ui_markers::TruckUI;
use untruck_core::types::tab::{TabContents, TabState};
use untruck_core::types::truck_button::TruckButtonType;

/// Trait to prevent CurrentDifficulty spilling to uncore
pub(crate) trait FromTab {
    fn from_tab(tab: TabContents, difficulty: &CurrentDifficulty) -> Self;
}

impl FromTab for TruckTab {
    /// Creates a new `TruckTab` from a `TabContents` enum.
    fn from_tab(tab: TabContents, difficulty: &CurrentDifficulty) -> Self {
        // Determine the default tab based on difficulty (tutorials show Journal)
        let is_tutorial = matches!(
            difficulty.0,
            undifficulty_core::difficulty::Difficulty::TutorialChapter1
                | undifficulty_core::difficulty::Difficulty::TutorialChapter2
        );
        let default_tab = if is_tutorial {
            TabContents::Journal
        } else {
            TabContents::Loadout
        };

        // Set the tab state based on whether this is the default tab
        let state = if tab == default_tab {
            TabState::Selected
        } else {
            tab.default_state()
        };
        Self {
            tabname: tab.name().to_owned(),
            state,
            contents: tab,
        }
    }
}

fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<UIPanelMaterial>>,
    game_state: Res<State<InGameUiState>>,
    truck_ui_assets: Res<TruckUiAssets>,
    gear_assets: Res<GearAssets>,
    difficulty: Res<CurrentDifficulty>, // Access the difficulty settings
    gear_registry: Res<GearSpawnerRegistry>,
    sprite_registry: Res<SpriteRegistry>,
) {
    const MARGIN_PERCENT: f32 = 0.5;
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );
    let init_vis = if *game_state == InGameUiState::Truck {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    // Client doesn't have a UI for equipment yet, so we spawn a dummy one to avoid panics.
    // In a real implementation we would have a read-only view of the truck.
    // However, the current UI handles interaction which only the host should do.

    type Cb<'a, 'b> = &'b mut ChildSpawnerCommands<'a>;
    let panel_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_PANEL_BGCOLOR.into(),
    });
    let sensors = |p: Cb| sensors::setup_sensors_ui(p, &truck_ui_assets);
    let left_column = |p: Cb| {
        p.spawn((
            MaterialNode(panel_material.clone()),
            Node {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::left(Val::Percent(MARGIN_PERCENT)),
                margin: MARGIN,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                flex_grow: 1.0,
                ..default()
            },
        ))
        .with_children(|p| sanity::setup_sanity_ui(p, &truck_ui_assets));

        p.spawn((
            MaterialNode(panel_material.clone()),
            Node {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::left(Val::Percent(MARGIN_PERCENT)),
                margin: MARGIN,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                flex_grow: 1.0,
                ..default()
            },
        ))
        .with_children(sensors);
    };
    let mid_column = |p: Cb| {
        let mut title_tab = |p: Cb, tab: TabContents| {
            // Directly use TruckTab from uncore, assuming its `from_tab` takes `&CurrentDifficulty`
            let truck_tab = TruckTab::from_tab(tab, &difficulty);
            let txt_fg = match truck_tab.state {
                TabState::Selected => colors::TRUCKUI_BGCOLOR.with_alpha(1.0),
                TabState::Pressed => colors::TRUCKUI_BGCOLOR.with_alpha(0.8),
                TabState::Hover => colors::TRUCKUI_ACCENT2_COLOR.with_alpha(0.6),
                TabState::Default => Hsla::from(colors::TRUCKUI_ACCENT_COLOR)
                    .with_saturation(0.1)
                    .with_alpha(0.6)
                    .into(),
                TabState::Disabled => colors::INVENTORY_STATS_COLOR.with_alpha(0.05),
            };
            let tab_bg = materials.add(UIPanelMaterial {
                color: match truck_tab.state {
                    TabState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                    TabState::Selected => colors::TRUCKUI_ACCENT_COLOR,
                    TabState::Hover => colors::TRUCKUI_BGCOLOR,
                    TabState::Default => colors::TRUCKUI_BGCOLOR.with_alpha(0.7),
                    TabState::Disabled => colors::TRUCKUI_BGCOLOR.with_alpha(0.5),
                }
                .into(),
            });
            let text = (
                Text::new(&truck_tab.tabname),
                TextFont {
                    font: truck_ui_assets.font_londrina_light.clone(),
                    font_size: 35.0 * FONT_SCALE,
                    ..default()
                },
                TextColor(txt_fg),
                TextLayout::default(),
                Node {
                    flex_grow: 0.5,
                    ..default()
                },
            );
            p.spawn((
                MaterialNode(tab_bg),
                Node {
                    padding: UiRect::new(
                        Val::Px(10.0 * UI_SCALE),
                        Val::Px(30.0 * UI_SCALE),
                        Val::ZERO,
                        Val::ZERO,
                    ),
                    margin: UiRect::new(
                        Val::Percent(MARGIN_PERCENT),
                        Val::Percent(MARGIN_PERCENT),
                        Val::Percent(MARGIN_PERCENT),
                        Val::ZERO,
                    ),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Interaction::None,
                truck_tab, // This component is uncore::components::truck_ui::TruckTab
            ))
            .with_children(|p| {
                p.spawn(Node {
                    flex_grow: 0.5,
                    flex_shrink: 1.0,
                    ..default()
                });
                p.spawn(text);
            });
        };

        p.spawn(Node {
            margin: UiRect::all(Val::ZERO),
            padding: UiRect::all(Val::ZERO),
            ..default()
        })
        .with_children(|p| {
            title_tab(p, TabContents::Loadout);
            title_tab(p, TabContents::LocationMap);
            title_tab(p, TabContents::CameraFeed);
            title_tab(p, TabContents::Journal);
        });
        p.spawn(Node {
            margin: UiRect::top(Val::Px(-4.1)),
            padding: UiRect::all(Val::ZERO),
            border: UiRect::all(Val::Px(1.50)),
            ..default()
        })
        .insert(BorderColor::all(colors::TRUCKUI_ACCENT_COLOR));

        let base_node = Node {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Percent(MARGIN_PERCENT),
            flex_grow: 1.0,
            flex_shrink: 0.0,
            ..default()
        };
        p.spawn((base_node.clone(), TabContents::Loadout))
            .with_children(|p| {
                loadoutui::setup_loadout_ui(
                    p,
                    &truck_ui_assets,
                    &gear_assets,
                    &mut materials,
                    &difficulty,
                    &gear_registry,
                    &sprite_registry,
                )
            });
        p.spawn((base_node.clone(), TabContents::Journal))
            .with_children(|p| journalui::setup_journal_ui(p, &truck_ui_assets, &difficulty));

        p.spawn(Node {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Percent(MARGIN_PERCENT),
            flex_grow: 1.0,
            ..default()
        });
    };
    let right_column = |p: Cb| {
        p.spawn((
            MaterialNode(panel_material.clone()),
            Node {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                margin: MARGIN,
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                flex_grow: 1.0,
                ..default()
            },
        ))
        .with_children(|p| activity::setup_activity_ui(p, &truck_ui_assets));

        p.spawn((
            Node {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                margin: MARGIN,
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                justify_content: JustifyContent::SpaceEvenly,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Percent(MARGIN_PERCENT),
                column_gap: Val::Percent(MARGIN_PERCENT),
                flex_grow: 0.01,
                ..default()
            },
            colors::DEBUG_BCOLOR,
        ))
        .with_children(|buttons| {
            buttons
                .spawn(Button)
                .insert(Node {
                    min_height: Val::Px(60.0 * UI_SCALE),
                    border: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Percent(MARGIN_PERCENT)),
                    position_type: PositionType::Relative,
                    ..default()
                })
                .insert(ZIndex(20))
                .insert(BackgroundColor(Color::NONE))
                .insert(BorderColor::all(Color::NONE))
                .insert(Interaction::None)
                .insert(TruckButtonType::ExitTruck.into_component())
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Exit Truck"),
                        TextFont {
                            font: truck_ui_assets.font_titillium_semibold.clone(),
                            font_size: 25.0 * FONT_SCALE,
                            ..default()
                        },
                        TextColor(colors::BUTTON_EXIT_TRUCK_TXTCOLOR),
                        TextLayout::default(),
                    ));
                });
            buttons
                .spawn(Button)
                .insert(Node {
                    min_height: Val::Px(60.0 * UI_SCALE),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(4.0 * UI_SCALE)),
                    position_type: PositionType::Relative,
                    ..default()
                })
                .insert(ZIndex(20))
                .insert(BackgroundColor(Color::NONE))
                .insert(BorderColor::all(Color::NONE))
                .insert(Interaction::None)
                .insert(TruckButtonType::EndMission.into_component())
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("End Mission"),
                        TextFont {
                            font: truck_ui_assets.font_titillium_semibold.clone(),
                            font_size: 25.0 * FONT_SCALE,
                            ..default()
                        },
                        TextColor(colors::BUTTON_END_MISSION_TXTCOLOR),
                        TextLayout::default(),
                    ));
                });
        });
    };
    let truck_ui = |p: Cb| {
        p.spawn((
            Node {
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(180.0 * UI_SCALE),
                min_height: Val::Px(10.0),
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_grow: 0.4,
                ..default()
            },
            colors::DEBUG_BCOLOR,
        ))
        .with_children(left_column);

        p.spawn((
            MaterialNode(panel_material.clone()),
            Node {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_basis: Val::Percent(55.0),
                flex_grow: 1.0,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(mid_column);

        p.spawn((
            Node {
                border: UiRect::all(Val::Px(1.0)),
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                justify_content: JustifyContent::FlexStart,
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_direction: FlexDirection::Column,
                flex_grow: 0.4,
                ..default()
            },
            colors::DEBUG_BCOLOR,
        ))
        .with_children(right_column);
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(98.0),
                height: Val::Percent(96.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Percent(MARGIN_PERCENT),
                padding: MARGIN,
                margin: MARGIN,
                ..default()
            },
            init_vis,
            BackgroundColor(colors::TRUCKUI_BGCOLOR),
        ))
        .insert(TruckUI)
        .with_children(truck_ui);
}

fn update_tab_interactions(
    mut materials: ResMut<Assets<UIPanelMaterial>>,
    mut qt: Query<(
        Ref<Interaction>,
        &mut TruckTab, // This is uncore::components::truck_ui::TruckTab
        &Children,
        &MaterialNode<UIPanelMaterial>,
    )>,
    mut qc: Query<(&mut Node, &TabContents)>, // This TabContents is uncore::components::truck_ui::TabContents
    mut text_query: Query<(&mut TextColor, &mut TextFont)>,
) {
    let mut new_selected_cnt = None;
    let mut changed = 0;
    for (int, _, _, _) in &qt {
        if !int.is_changed() {
            continue;
        }
        changed += 1;
    }
    for (int, tt, _, _) in &qt {
        if !int.is_changed() {
            continue;
        }
        let int_val = *int.into_inner();
        if tt.state == TabState::Pressed && int_val == Interaction::Hovered {
            new_selected_cnt = Some(tt.contents.clone());
        }
        if changed > 1 && tt.state == TabState::Selected {
            new_selected_cnt = Some(tt.contents.clone());
        }
    }
    let new_selection = new_selected_cnt.is_some();
    for (int, mut tt, children, panmat) in &mut qt {
        if !int.is_changed() && !new_selection {
            continue;
        }
        let int_val = *int.into_inner();

        if tt.state == TabState::Selected && new_selection && changed <= 1 {
            tt.state = TabState::Default;
        } else if tt.state == TabState::Pressed && int_val == Interaction::Hovered {
            tt.state = TabState::Selected;
        } else {
            match tt.state {
                TabState::Disabled | TabState::Selected => {}
                TabState::Default | TabState::Hover | TabState::Pressed => {
                    tt.state = match int_val {
                        Interaction::Pressed => TabState::Pressed,
                        Interaction::Hovered => TabState::Hover,
                        Interaction::None => TabState::Default,
                    };
                }
            }
        }
        let (mut textcolor, mut textfont) = text_query.get_mut(children[1]).unwrap();
        textcolor.0 = match tt.state {
            TabState::Selected => colors::TRUCKUI_BGCOLOR.with_alpha(1.0),
            TabState::Pressed => colors::TRUCKUI_BGCOLOR.with_alpha(0.8),
            TabState::Hover => colors::TRUCKUI_ACCENT2_COLOR.with_alpha(0.6),
            TabState::Default => Hsla::from(colors::TRUCKUI_ACCENT_COLOR)
                .with_saturation(0.1)
                .with_alpha(0.6)
                .into(),
            TabState::Disabled => colors::INVENTORY_STATS_COLOR.with_alpha(0.05),
        };
        textfont.font_size = match tt.state {
            TabState::Selected => 35.0 * FONT_SCALE,
            _ => 24.0 * FONT_SCALE,
        };
        if let Some(mat) = materials.get_mut(panmat) {
            mat.color = match tt.state {
                TabState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
                TabState::Selected => colors::TRUCKUI_ACCENT_COLOR,
                TabState::Hover => colors::TRUCKUI_BGCOLOR,
                TabState::Default => colors::TRUCKUI_BGCOLOR.with_alpha(0.7),
                TabState::Disabled => colors::TRUCKUI_BGCOLOR.with_alpha(0.5),
            }
            .into();
        } else {
            warn!("Material not found for TruckTab update.");
        }
    }
    if let Some(cnt) = new_selected_cnt {
        for (mut style, tc) in &mut qc {
            let new_dis = match cnt == *tc {
                true => Display::Flex,
                false => Display::None,
            };
            if new_dis != style.display {
                style.display = new_dis;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), setup_ui)
        .add_systems(
            Update,
            update_tab_interactions.run_if(in_state(InGameUiState::Truck)),
        );
}
