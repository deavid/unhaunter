pub mod activity;
pub mod journal;
pub mod sanity;
pub mod sensors;
pub mod uibutton;

use bevy::app::App;
use bevy::prelude::*;

use crate::colors;
use crate::game::GameConfig;
use crate::gear::playergear::PlayerGear;
use crate::player::PlayerSprite;
use crate::truck::uibutton::TruckButtonType;
use crate::{
    ghost_definitions::{self, GhostType},
    materials::{self, UIPanelMaterial},
    root,
};

#[derive(Component, Debug)]
pub struct TruckUI;

#[derive(Clone, Debug, Event)]
pub enum TruckUIEvent {
    EndMission,
    ExitTruck,
    CraftRepellent,
}

#[derive(Component, Debug)]
pub struct TruckUIGhostGuess;

#[derive(Debug, Resource, Default)]
pub struct GhostGuess {
    pub ghost_type: Option<GhostType>,
}

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

        // Journal contents
        p.spawn(NodeBundle {
            border_color: colors::TRUCKUI_ACCENT_COLOR.into(),
            style: Style {
                margin: UiRect::top(Val::Px(-3.0)),
                padding: UiRect::all(Val::ZERO),
                border: UiRect::all(Val::Px(1.50)),
                ..default()
            },
            ..default()
        });

        p.spawn(
            TextBundle::from_section(
                "Select evidence:",
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

        // Evidence selection
        p.spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                // flex_direction: FlexDirection::Row,
                // flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(4.0),
                column_gap: Val::Px(4.0),
                display: Display::Grid,
                grid_template_columns: vec![
                    GridTrack::auto(),
                    GridTrack::auto(),
                    GridTrack::auto(),
                    GridTrack::auto(),
                ],
                grid_template_rows: vec![GridTrack::auto(), GridTrack::auto()],

                ..default()
            },
            ..default()
        })
        .with_children(|evblock| {
            for evidence in ghost_definitions::Evidence::all() {
                evblock
                    .spawn(ButtonBundle {
                        style: Style {
                            min_height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(0.9)),
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,
                            display: Display::Grid,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                            padding: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(TruckButtonType::Evidence(evidence).into_component())
                    .with_children(|btn| {
                        btn.spawn(TextBundle::from_section(
                            evidence.name(),
                            TextStyle {
                                font: handles.fonts.titillium.w400_regular.clone(),
                                font_size: 22.0,
                                ..default()
                            },
                        ));
                    });
            }
        });
        // ---- Ghost guess
        p.spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::End,
                column_gap: Val::Percent(MARGIN_PERCENT),
                flex_basis: Val::Px(50.0),
                flex_grow: 0.5,
                flex_shrink: 0.0,
                ..default()
            },
            ..default()
        })
        .with_children(|guess| {
            guess.spawn(
                TextBundle::from_section(
                    "Possible ghost with the selected evidence:",
                    TextStyle {
                        font: handles.fonts.chakra.w300_light.clone(),
                        font_size: 25.0,
                        color: colors::TRUCKUI_TEXT_COLOR,
                    },
                )
                .with_style(Style {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    ..default()
                }),
            );
        });

        // Ghost selection
        p.spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                // row_gap: Val::Px(4.0),
                // column_gap: Val::Px(4.0),
                display: Display::Grid,
                grid_template_columns: vec![
                    GridTrack::auto(),
                    GridTrack::auto(),
                    GridTrack::auto(),
                    GridTrack::auto(),
                ],
                grid_auto_rows: GridTrack::flex(1.0),
                flex_grow: 1.0,
                ..default()
            },
            background_color: colors::TRUCKUI_BGCOLOR.into(),
            ..default()
        })
        .with_children(|ghost_selection| {
            for ghost_type in ghost_definitions::GhostType::all() {
                ghost_selection
                    .spawn(ButtonBundle {
                        style: Style {
                            min_height: Val::Px(20.0),
                            border: UiRect::all(Val::Px(0.9)),
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::new(
                                Val::Px(5.0),
                                Val::Px(2.0),
                                Val::Px(0.0),
                                Val::Px(2.0),
                            ),
                            display: Display::Grid,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(TruckButtonType::Ghost(ghost_type).into_component())
                    .with_children(|btn| {
                        btn.spawn(TextBundle::from_section(
                            ghost_type.name(),
                            TextStyle {
                                font: handles.fonts.titillium.w400_regular.clone(),
                                font_size: 22.0,
                                ..default()
                            },
                        ));
                    });
            }
        });
        // --- Ghost selected
        p.spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::End,
                column_gap: Val::Percent(MARGIN_PERCENT),
                flex_basis: Val::Px(50.0),
                flex_grow: 0.5,
                flex_shrink: 0.0,
                ..default()
            },

            ..default()
        })
        .with_children(|guess| {
            guess.spawn(
                TextBundle::from_section(
                    "With the above evidence we believe the ghost is:",
                    TextStyle {
                        font: handles.fonts.chakra.w300_light.clone(),
                        font_size: 25.0,
                        color: colors::TRUCKUI_TEXT_COLOR,
                    },
                )
                .with_style(Style {
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    ..default()
                }),
            );
            let ghost_guess = TextBundle::from_section(
                "-- Unknown --",
                TextStyle {
                    font: handles.fonts.titillium.w600_semibold.clone(),
                    font_size: 28.0,
                    color: colors::TRUCKUI_TEXT_COLOR,
                },
            );
            guess
                .spawn(NodeBundle {
                    background_color: colors::TRUCKUI_BGCOLOR.into(),
                    style: Style {
                        padding: UiRect::all(Val::Px(4.0)),
                        flex_basis: Val::Px(300.0),
                        flex_grow: 0.0,
                        flex_shrink: 0.0,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|node| {
                    node.spawn(ghost_guess).insert(TruckUIGhostGuess);
                });
        });

        // ---- Synthesis of Unhaunter essence
        p.spawn(ButtonBundle {
            style: Style {
                min_height: Val::Px(30.0),
                width: Val::Percent(50.0),
                border: MARGIN,
                align_self: AlignSelf::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: MARGIN,
                ..default()
            },
            ..default()
        })
        .insert(TruckButtonType::CraftRepellent.into_component())
        .with_children(|btn| {
            btn.spawn(TextBundle::from_section(
                "Craft Unhaunter™ Ghost Repellent",
                TextStyle {
                    font: handles.fonts.titillium.w600_semibold.clone(),
                    font_size: 32.0,
                    ..default()
                },
            ));
        });

        // ----
        p.spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Percent(MARGIN_PERCENT),
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        });
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

pub fn cleanup(mut commands: Commands, qtui: Query<Entity, With<TruckUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn show_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Inherited;
    }
}

pub fn hide_ui(mut qtui: Query<&mut Visibility, With<TruckUI>>) {
    for mut v in qtui.iter_mut() {
        *v = Visibility::Hidden;
    }
}

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != root::GameState::Truck {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::None);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn truckui_event_handle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_truckui: EventReader<TruckUIEvent>,
    mut next_state: ResMut<NextState<root::State>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    gg: Res<GhostGuess>,
    gc: Res<GameConfig>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
) {
    for ev in ev_truckui.read() {
        match ev {
            TruckUIEvent::EndMission => {
                game_next_state.set(root::GameState::None);
                next_state.set(root::State::Summary);
            }
            TruckUIEvent::ExitTruck => game_next_state.set(root::GameState::None),
            TruckUIEvent::CraftRepellent => {
                for (player, mut gear) in q_gear.iter_mut() {
                    if player.id == gc.player_id {
                        if let Some(ghost_type) = gg.ghost_type {
                            gear.craft_repellent(ghost_type);
                            commands.spawn(AudioBundle {
                                source: asset_server.load("sounds/effects-dingdingding.ogg"),
                                settings: PlaybackSettings {
                                    mode: bevy::audio::PlaybackMode::Despawn,
                                    volume: bevy::audio::Volume::new(1.0),
                                    speed: 1.0,
                                    paused: false,
                                    spatial: false,
                                    spatial_scale: None,
                                },
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::InGame), setup_ui)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(OnEnter(root::GameState::Truck), show_ui)
        .add_systems(OnExit(root::GameState::Truck), hide_ui)
        .add_event::<TruckUIEvent>()
        .init_resource::<GhostGuess>()
        .add_systems(Update, keyboard)
        .add_systems(Update, journal::ghost_guess_system)
        .add_systems(
            FixedUpdate,
            (journal::button_system, sanity::update_sanity)
                .run_if(in_state(root::GameState::Truck)),
        )
        .add_systems(Update, truckui_event_handle);
}
