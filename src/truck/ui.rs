use bevy::prelude::*;

use crate::colors;
use crate::truck::uibutton::TruckButtonType;
use crate::truck::{activity, journalui, sanity, sensors, TruckUI};
use crate::{
    materials::{self, UIPanelMaterial},
    root,
};

pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<materials::UIPanelMaterial>>,
    handles: Res<root::GameAssets>,
) {
    // Load Truck UI
    const MARGIN_PERCENT: f32 = 0.5;
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );
    type Cb<'a, 'b> = &'b mut ChildBuilder<'a>;

    let panel_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_PANEL_BGCOLOR,
    });
    let tab_selected_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_ACCENT_COLOR,
    });
    let tab_hover_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_BGCOLOR,
    });
    let tab_default_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_BGCOLOR.with_a(0.7),
    });
    let tab_disabled_material = materials.add(UIPanelMaterial {
        color: colors::TRUCKUI_BGCOLOR.with_a(0.5),
    });

    let sensors = |p: Cb| sensors::setup_sensors_ui(p, &handles);

    let left_column = |p: Cb| {
        // Top Left - Sanity
        p.spawn(MaterialNodeBundle {
            material: panel_material.clone(),

            style: Style {
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
            ..default()
        })
        .with_children(|p| sanity::setup_sanity_ui(p, &handles));
        // Bottom Left - Sensors
        p.spawn(MaterialNodeBundle {
            material: panel_material.clone(),

            style: Style {
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
            ..default()
        })
        .with_children(sensors);
    };

    let mid_column = |p: Cb| {
        enum TabState {
            Selected,
            Hover,
            Default,
            Disabled,
        }
        let title_tab = |p: Cb, txt: &str, state: TabState| {
            let tab_bg = match state {
                TabState::Selected => tab_selected_material.clone(),
                TabState::Hover => tab_hover_material.clone(),
                TabState::Default => tab_default_material.clone(),
                TabState::Disabled => tab_disabled_material.clone(),
            };
            let txt_fg = match state {
                TabState::Selected => colors::TRUCKUI_BGCOLOR.with_a(1.0),
                TabState::Hover => colors::TRUCKUI_ACCENT2_COLOR.with_a(0.6),
                TabState::Default => colors::TRUCKUI_ACCENT_COLOR.with_s(0.1).with_a(0.6),
                TabState::Disabled => colors::INVENTORY_STATS_COLOR.with_a(0.05),
            };

            let text = TextBundle::from_section(
                txt,
                TextStyle {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 35.0,
                    color: txt_fg,
                },
            )
            .with_style(Style {
                height: Val::Px(40.0),
                ..default()
            });
            p.spawn(MaterialNodeBundle {
                material: tab_bg,
                style: Style {
                    padding: UiRect::new(Val::Px(10.0), Val::Px(30.0), Val::ZERO, Val::ZERO),
                    margin: UiRect::new(
                        Val::Percent(MARGIN_PERCENT),
                        Val::Percent(MARGIN_PERCENT),
                        Val::Percent(MARGIN_PERCENT),
                        Val::ZERO,
                    ),
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 0.0,
                    ..default()
                },
                ..default()
            })
            .with_children(|p| {
                p.spawn(text);
            });
        };

        // Tab titles:
        p.spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::ZERO),
                padding: UiRect::all(Val::ZERO),
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            title_tab(p, "Loadout", TabState::Hover);
            title_tab(p, "Location Map", TabState::Default);
            title_tab(p, "Camera Feed", TabState::Disabled);
            title_tab(p, "Journal", TabState::Selected);
        });

        journalui::setup_journal_ui(p, &handles);
    };

    let right_column = |p: Cb| {
        // Top Right - Activity
        p.spawn(MaterialNodeBundle {
            material: panel_material.clone(),

            style: Style {
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
            ..default()
        })
        .with_children(|p| activity::setup_activity_ui(p, &handles));
        // Bottom Right - 2 buttons - Exit Truck + End mission.
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,

            style: Style {
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
            ..default()
        })
        .with_children(|buttons| {
            buttons
                .spawn(ButtonBundle {
                    style: Style {
                        min_height: Val::Px(60.0),
                        border: MARGIN,
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Percent(MARGIN_PERCENT)),
                        ..default()
                    },
                    ..default()
                })
                .insert(TruckButtonType::ExitTruck.into_component())
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "Exit Truck",
                        TextStyle {
                            font: handles.fonts.titillium.w600_semibold.clone(),
                            font_size: 35.0,
                            ..default()
                        },
                    ));
                });
            buttons
                .spawn(ButtonBundle {
                    style: Style {
                        min_height: Val::Px(60.0),
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        border: MARGIN,
                        ..default()
                    },
                    ..default()
                })
                .insert(TruckButtonType::EndMission.into_component())
                .with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "End Mission",
                        TextStyle {
                            font: handles.fonts.titillium.w600_semibold.clone(),
                            font_size: 35.0,
                            ..default()
                        },
                    ));
                });
        });
    };

    let truck_ui = |p: Cb| {
        // Left column
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,
            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(180.0),
                min_height: Val::Px(10.0),
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_grow: 0.4,
                ..default()
            },
            ..default()
        })
        .with_children(left_column);
        // Mid content
        p.spawn(MaterialNodeBundle {
            material: panel_material.clone(),

            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(1.0)),
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        })
        .with_children(mid_column);
        // Right column
        p.spawn(NodeBundle {
            border_color: colors::DEBUG_BCOLOR,

            style: Style {
                border: UiRect::all(Val::Px(1.0)),
                min_width: Val::Px(10.0),
                min_height: Val::Px(10.0),
                justify_content: JustifyContent::FlexStart,
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_direction: FlexDirection::Column,
                flex_grow: 0.4,
                ..default()
            },
            ..default()
        })
        .with_children(right_column);
    };

    commands
        .spawn(NodeBundle {
            background_color: colors::TRUCKUI_BGCOLOR.into(),

            style: Style {
                width: Val::Percent(98.0),
                height: Val::Percent(96.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Percent(MARGIN_PERCENT),
                padding: MARGIN,
                margin: MARGIN,
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(TruckUI)
        .with_children(truck_ui);

    // ---
}
