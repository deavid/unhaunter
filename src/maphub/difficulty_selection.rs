use bevy::prelude::*;

use crate::colors;
use crate::difficulty::{CurrentDifficulty, Difficulty};
use crate::game::level::LoadLevelEvent;
use crate::platform::plt::UI_SCALE;
use crate::root;

use super::MapHubState;
use crate::maphub::map_selection::MapSelectedEvent; // Add MapSelectedEvent

#[derive(Component, Debug)]
pub struct DifficultySelectionUI;

#[derive(Component, Debug)]
pub struct DifficultyDescriptionUI;

#[derive(Component, Debug)]
pub struct DifficultySelectionItem {
    pub difficulty: Difficulty,
}

#[derive(Resource, Debug, Default)]
pub struct DifficultySelectionState {
    pub selected_difficulty: Difficulty,
    pub selected_map_idx: usize, // Add this to store the selected map index
}

// New event for confirming difficulty selection
#[derive(Debug, Clone, Event)]
pub struct DifficultyConfirmedEvent;

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(MapHubState::DifficultySelection), setup_systems)
        .add_systems(OnExit(MapHubState::DifficultySelection), cleanup_systems)
        .add_systems(
            Update,
            (keyboard, handle_difficulty_selection, update_item_colors)
                .run_if(in_state(MapHubState::DifficultySelection)),
        )
        .add_event::<DifficultyConfirmedEvent>(); // Register the new event
}

pub fn setup_systems(
    mut commands: Commands,
    mut ev_map_selected: EventReader<MapSelectedEvent>, // Access MapSelectedEvent
    handles: Res<root::GameAssets>,
) {
    // Create the UI for the difficulty selection screen
    setup_ui(&mut commands, &handles);

    // Initialize the DifficultySelectionState resource
    let mut difficulty_selection_state = DifficultySelectionState::default();

    // Get the selected map index from the MapSelectedEvent
    if let Some(event) = ev_map_selected.read().next() {
        difficulty_selection_state.selected_map_idx = event.map_idx;
    }

    commands.insert_resource(difficulty_selection_state);
}

pub fn cleanup_systems(mut commands: Commands, qtui: Query<Entity, With<DifficultySelectionUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

// Handle the DifficultyConfirmedEvent
pub fn handle_difficulty_selection(
    mut ev_difficulty_confirmed: EventReader<DifficultyConfirmedEvent>,
    mut next_state: ResMut<NextState<MapHubState>>,
    mut difficulty: ResMut<CurrentDifficulty>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    mut ev_load_level: EventWriter<LoadLevelEvent>,
    maps: Res<root::Maps>,
) {
    for _ in ev_difficulty_confirmed.read() {
        // Set the selected difficulty in the CurrentDifficulty resource
        difficulty.0 = difficulty_selection_state
            .selected_difficulty
            .create_difficulty_struct();

        // Get the selected map's filepath
        let map_filepath = maps.maps[difficulty_selection_state.selected_map_idx]
            .path
            .clone();

        // Send the LoadLevelEvent to trigger map loading
        ev_load_level.send(LoadLevelEvent { map_filepath });
        next_state.set(MapHubState::None);
    }
}

pub fn keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut difficulty_selection_state: ResMut<DifficultySelectionState>,
    mut ev_difficulty_confirmed: EventWriter<DifficultyConfirmedEvent>,
    mut next_state: ResMut<NextState<MapHubState>>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        for _ in 0..4 {
            difficulty_selection_state.selected_difficulty =
                difficulty_selection_state.selected_difficulty.prev();
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        for _ in 0..4 {
            difficulty_selection_state.selected_difficulty =
                difficulty_selection_state.selected_difficulty.next();
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        difficulty_selection_state.selected_difficulty =
            difficulty_selection_state.selected_difficulty.prev();
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        difficulty_selection_state.selected_difficulty =
            difficulty_selection_state.selected_difficulty.next();
    } else if keyboard_input.just_pressed(KeyCode::Enter) {
        // Send the confirmation event only if a valid difficulty is selected
        ev_difficulty_confirmed.send(DifficultyConfirmedEvent);
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        // Transition back to the map selection screen
        next_state.set(MapHubState::MapSelection);
    }
}

// Update item colors based on selected difficulty
pub fn update_item_colors(
    mut q_items: Query<(&DifficultySelectionItem, &Children)>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    mut q_text: Query<&mut Text>,
    q_desc: Query<Entity, With<DifficultyDescriptionUI>>,
) {
    if !difficulty_selection_state.is_changed() {
        return;
    }
    let sel_difficulty = difficulty_selection_state.selected_difficulty;
    for (difficulty_item, children) in q_items.iter_mut() {
        if let Ok(mut text) = q_text.get_mut(children[0]) {
            for section in text.sections.iter_mut() {
                let new_color = if difficulty_item.difficulty == sel_difficulty {
                    colors::MENU_ITEM_COLOR_ON
                } else {
                    colors::MENU_ITEM_COLOR_OFF
                };
                if new_color != section.style.color {
                    section.style.color = new_color;
                }
            }
        }
    }
    let dif = sel_difficulty.create_difficulty_struct();
    for entity in q_desc.iter() {
        if let Ok(mut text) = q_text.get_mut(entity) {
            for section in text.sections.iter_mut() {
                let name = &dif.difficulty_name;
                let description = &dif.difficulty_description;
                let score_mult = dif.difficulty_score_multiplier;

                let text = [
                    format!("Difficulty <{name}>: {description}"),
                    format!("Score Bonus: {score_mult:.2}x"),
                ];
                section.value = text.join("\n");
            }
        }
    }
}

pub fn setup_ui(commands: &mut Commands, handles: &root::GameAssets) {
    const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;

    let main_color = Color::Srgba(Srgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: Val::Percent(10.0),
                    right: Val::Percent(10.0),
                    top: Val::Percent(5.0),
                    bottom: Val::Percent(5.0),
                },
                ..default()
            },
            ..default()
        })
        .insert(DifficultySelectionUI)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(20.0),
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(64.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // logo
                    parent.spawn(ImageBundle {
                        style: Style {
                            aspect_ratio: Some(130.0 / 17.0),
                            width: Val::Percent(80.0),
                            height: Val::Auto,
                            max_width: Val::Percent(80.0),
                            max_height: Val::Percent(100.0),
                            flex_shrink: 1.0,
                            ..default()
                        },
                        image: handles.images.title.clone().into(),
                        ..default()
                    });
                });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    ..default()
                },
                ..default()
            });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(60.0),
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: main_color.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // text
                    parent.spawn(TextBundle::from_section(
                        "Map Hub - Select Difficulty",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0,
                            color: Color::WHITE,
                        },
                    ));
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(10.0),
                            ..default()
                        },
                        ..default()
                    });
                    // Difficulty buttons in a 3-column grid
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(80.0),
                                justify_content: JustifyContent::SpaceEvenly,
                                align_items: AlignItems::Center,
                                display: Display::Grid,
                                grid_template_columns: RepeatedGridTrack::flex(4, 1.0), // 4 equal columns
                                grid_auto_rows: GridTrack::auto(),
                                row_gap: Val::Px(10.0),
                                column_gap: Val::Px(20.0),
                                ..default()
                            },
                            background_color: main_color.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            for difficulty in Difficulty::all() {
                                parent
                                    .spawn(ButtonBundle {
                                        style: Style {
                                            min_height: Val::Px(30.0),
                                            border: UiRect::all(Val::Px(0.9)),
                                            align_content: AlignContent::Center,
                                            justify_content: JustifyContent::Center,
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                                            ..default()
                                        },
                                        background_color: Color::NONE.into(), // Remove background color
                                        ..default()
                                    })
                                    .insert(DifficultySelectionItem { difficulty })
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            difficulty.difficulty_name(),
                                            TextStyle {
                                                font: handles.fonts.londrina.w300_light.clone(),
                                                font_size: 28.0 * UI_SCALE, // Reduced font size
                                                color: colors::MENU_ITEM_COLOR_OFF,
                                            },
                                        ));
                                    });
                            }
                        });
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(10.0),
                            ..default()
                        },
                        ..default()
                    });
                    // parent
                    //     .spawn(ButtonBundle {
                    //         style: Style {
                    //             min_height: Val::Px(30.0),
                    //             border: UiRect::all(Val::Px(0.9)),
                    //             align_content: AlignContent::Center,
                    //             justify_content: JustifyContent::Center,
                    //             flex_direction: FlexDirection::Column,
                    //             align_items: AlignItems::Center,
                    //             margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                    //             ..default()
                    //         },
                    //         background_color: Color::NONE.into(), // Remove background color
                    //         ..default()
                    //     })
                    //     .with_children(|btn| {
                    //         btn.spawn(TextBundle::from_section(
                    //             "Go Back",
                    //             TextStyle {
                    //                 font: handles.fonts.londrina.w300_light.clone(),
                    //                 font_size: 38.0,
                    //                 color: colors::MENU_ITEM_COLOR_OFF, // Default text color
                    //             },
                    //         ));
                    //     });
                });
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    ..default()
                },
                ..default()
            });
            parent
                .spawn(
                    TextBundle::from_section(
                        "Difficulty <>: Description",
                        TextStyle {
                            font: handles.fonts.titillium.w300_light.clone(),
                            font_size: 26.0 * UI_SCALE,
                            color: colors::MENU_ITEM_COLOR_OFF,
                        },
                    )
                    .with_style(Style {
                        padding: UiRect::all(Val::Percent(5.0 * UI_SCALE)),
                        align_content: AlignContent::Center,
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        justify_self: JustifySelf::Center,
                        flex_grow: 0.0,
                        flex_shrink: 0.0,
                        flex_basis: Val::Px(155.0 * UI_SCALE),
                        max_height: Val::Px(155.0 * UI_SCALE),
                        ..default()
                    }),
                )
                .insert(DifficultyDescriptionUI);
        });
    info!("MapHub - Difficulty menu loaded");
}
