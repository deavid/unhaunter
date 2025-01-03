use uncore::colors;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};

use crate::difficulty::CurrentDifficulty;
use crate::truck::uibutton::TruckButtonType;
use crate::truck::{activity, journalui, loadoutui, sanity, sensors, TruckUI};
use crate::{uncore_materials::UIPanelMaterial, root};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents the visual state of a tab in the truck UI.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabState {
    /// The tab is currently selected and active.
    Selected,
    /// The tab is being pressed.
    Pressed,
    /// The mouse is hovering over the tab.
    Hover,
    /// The tab is in its default, unselected state.
    #[default]
    Default,
    /// The tab is disabled and cannot be interacted with.
    Disabled,
}

/// Represents the different content sections within the truck UI.
#[derive(Debug, Clone, Component, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabContents {
    /// The loadout tab for managing player gear.
    Loadout,
    /// The location map tab. (Currently disabled)
    LocationMap,
    /// The camera feed tab. (Currently disabled)
    CameraFeed,
    /// The journal tab for reviewing evidence and guessing the ghost type.
    Journal,
}

impl TabContents {
    /// Returns the display name for the tab content.
    pub fn name(&self) -> &'static str {
        match self {
            TabContents::Loadout => "Loadout",
            TabContents::LocationMap => "Location Map",
            TabContents::CameraFeed => "Camera Feed",
            TabContents::Journal => "Journal",
        }
    }

    /// Returns the default `TabState` for the tab content.
    pub fn default_state(&self) -> TabState {
        match self {
            TabContents::Loadout => TabState::Default,
            TabContents::LocationMap => TabState::Disabled,
            TabContents::CameraFeed => TabState::Disabled,
            TabContents::Journal => TabState::Default,
        }
    }
}

/// Represents a tab in the truck UI.
#[derive(Debug, Clone, Component)]
pub struct TruckTab {
    /// The display name of the tab.
    pub tabname: String,
    /// The current visual state of the tab.
    pub state: TabState,
    /// The content section associated with the tab.
    pub contents: TabContents,
}

impl TruckTab {
    /// Creates a new `TruckTab` from a `TabContents` enum.
    pub fn from_tab(tab: TabContents, difficulty: &CurrentDifficulty) -> Self {
        // Set the tab state based on difficulty
        let state = if tab == difficulty.0.default_van_tab {
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

    /// Updates the tab's visual state based on the given interaction.
    pub fn update_from_interaction(&mut self, interaction: &Interaction) {
        match self.state {
            TabState::Disabled | TabState::Selected => {}
            TabState::Default | TabState::Hover | TabState::Pressed => {
                self.state = match interaction {
                    Interaction::Pressed => TabState::Pressed,
                    Interaction::Hovered => TabState::Hover,
                    Interaction::None => TabState::Default,
                };
            }
        }
    }

    pub fn text_color(&self) -> Color {
        match self.state {
            TabState::Selected => colors::TRUCKUI_BGCOLOR.with_alpha(1.0),
            TabState::Pressed => colors::TRUCKUI_BGCOLOR.with_alpha(0.8),
            TabState::Hover => colors::TRUCKUI_ACCENT2_COLOR.with_alpha(0.6),
            TabState::Default => Hsla::from(colors::TRUCKUI_ACCENT_COLOR)
                .with_saturation(0.1)
                .with_alpha(0.6)
                .into(),
            TabState::Disabled => colors::INVENTORY_STATS_COLOR.with_alpha(0.05),
        }
    }

    pub fn bg_color(&self) -> Color {
        match self.state {
            TabState::Pressed => colors::TRUCKUI_ACCENT2_COLOR,
            TabState::Selected => colors::TRUCKUI_ACCENT_COLOR,
            TabState::Hover => colors::TRUCKUI_BGCOLOR,
            TabState::Default => colors::TRUCKUI_BGCOLOR.with_alpha(0.7),
            TabState::Disabled => colors::TRUCKUI_BGCOLOR.with_alpha(0.5),
        }
    }

    pub fn font_size(&self) -> f32 {
        match self.state {
            TabState::Selected => 35.0 * FONT_SCALE,
            _ => 24.0 * FONT_SCALE,
        }
    }
}

pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<UIPanelMaterial>>,
    handles: Res<root::GameAssets>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    const MARGIN_PERCENT: f32 = 0.5;
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );

    // Load Truck UI
    type Cb<'a, 'b> = &'b mut ChildBuilder<'a>;
    let panel_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_PANEL_BGCOLOR.into(),
    });
    let sensors = |p: Cb| sensors::setup_sensors_ui(p, &handles);
    let left_column = |p: Cb| {
        // Top Left - Sanity
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
        .with_children(|p| sanity::setup_sanity_ui(p, &handles));

        // Bottom Left - Sensors
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
            let truck_tab = TruckTab::from_tab(tab, &difficulty);
            let txt_fg = truck_tab.text_color();
            let tab_bg = materials.add(UIPanelMaterial {
                color: truck_tab.bg_color().into(),
            });
            let text = (
                Text::new(&truck_tab.tabname),
                TextFont {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 35.0 * FONT_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
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
                truck_tab,
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

        // Tab titles:
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
        .insert(BorderColor(colors::TRUCKUI_ACCENT_COLOR));

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
                loadoutui::setup_loadout_ui(p, &handles, &mut materials, &difficulty)
            });
        p.spawn((base_node.clone(), TabContents::Journal))
            .with_children(|p| journalui::setup_journal_ui(p, &handles, &difficulty));

        // ---
        p.spawn(Node {
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Percent(MARGIN_PERCENT),
            flex_grow: 1.0,
            ..default()
        });
    };
    let right_column = |p: Cb| {
        // Top Right - Activity
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
        .with_children(|p| activity::setup_activity_ui(p, &handles));

        // Bottom Right - 2 buttons - Exit Truck + End mission.
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
                    border: MARGIN,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Percent(MARGIN_PERCENT)),
                    ..default()
                })
                .insert(BackgroundColor(Color::NONE))
                .insert(BorderColor(Color::NONE))
                .insert(Interaction::None)
                .insert(TruckButtonType::ExitTruck.into_component())
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Exit Truck"),
                        TextFont {
                            font: handles.fonts.titillium.w600_semibold.clone(),
                            font_size: 25.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
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
                    border: MARGIN,
                    ..default()
                })
                .insert(BackgroundColor(Color::NONE))
                .insert(BorderColor(Color::NONE))
                .insert(Interaction::None)
                .insert(TruckButtonType::EndMission.into_component())
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("End Mission"),
                        TextFont {
                            font: handles.fonts.titillium.w600_semibold.clone(),
                            font_size: 25.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(colors::BUTTON_END_MISSION_TXTCOLOR),
                        TextLayout::default(),
                    ));
                });
        });
    };
    let truck_ui = |p: Cb| {
        // Left column
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

        // Mid content
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

        // Right column
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
            Visibility::Hidden,
            BackgroundColor(colors::TRUCKUI_BGCOLOR),
        ))
        .insert(TruckUI)
        .with_children(truck_ui);
    // ---
}

/// Updates the visual appearance and behavior of tabs in the truck UI.
///
/// This system handles:
///
/// * Changing the visual state of tabs based on mouse interactions (hover, press).
///
/// * Switching between different content sections when tabs are clicked.
///
/// * Updating the colors and styles of tabs to reflect their current state.
pub fn update_tab_interactions(
    mut materials: ResMut<Assets<UIPanelMaterial>>,
    mut qt: Query<(
        Ref<Interaction>,
        &mut TruckTab,
        &Children,
        &MaterialNode<UIPanelMaterial>,
    )>,
    mut qc: Query<(&mut Node, &TabContents)>,
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
        let int = *int.into_inner();
        if tt.state == TabState::Pressed && int == Interaction::Hovered {
            new_selected_cnt = Some(tt.contents.clone());
        }
        if changed > 1 && tt.state == TabState::Selected {
            // For the initialization pass
            new_selected_cnt = Some(tt.contents.clone());
        }
    }
    let new_selection = new_selected_cnt.is_some();
    for (int, mut tt, children, panmat) in &mut qt {
        if !int.is_changed() && !new_selection {
            continue;
        }
        let int = *int.into_inner();

        // warn!("Truck Tab {:?} - Interaction: {:?}", tt, int);
        if tt.state == TabState::Selected && new_selection && changed <= 1 {
            tt.state = TabState::Default;
        } else if tt.state == TabState::Pressed && int == Interaction::Hovered {
            tt.state = TabState::Selected;
        } else {
            tt.update_from_interaction(&int);
        }
        let (mut textcolor, mut textfont) = text_query.get_mut(children[1]).unwrap();
        textcolor.0 = tt.text_color();
        textfont.font_size = tt.font_size();
        let mat = materials.get_mut(panmat).unwrap();
        mat.color = tt.bg_color().into();
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
