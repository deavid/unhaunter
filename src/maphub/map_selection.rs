use bevy::prelude::*;

use crate::colors;
use crate::platform::plt::UI_SCALE;
use crate::root::Maps;
use crate::{maphub::MapHubState, root};

#[derive(Component, Debug)]
pub struct MapSelectionUI;

#[derive(Component, Debug)]
pub struct MapSelectionItem {
    pub map_idx: usize,
}

#[derive(Debug, Clone, Event)]
pub struct MapSelectedEvent {
    pub map_idx: usize,
}

#[derive(Resource, Debug, Default)]
pub struct MapSelectionState {
    pub selected_map_idx: usize,
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(MapHubState::MapSelection), setup_systems)
        .add_systems(OnExit(MapHubState::MapSelection), cleanup_systems)
        .add_systems(
            Update,
            (keyboard, update_item_colors).run_if(in_state(MapHubState::MapSelection)),
        )
        .init_resource::<MapSelectionState>()
        .add_event::<MapSelectedEvent>();
}

pub fn setup_systems(mut commands: Commands, maps: Res<Maps>, handles: Res<root::GameAssets>) {
    // Create the UI for the map selection screen
    setup_ui(&mut commands, &handles, &maps);

    // Add the MapSelectionItem component to the first map button
    commands
        .spawn(MapSelectionItem { map_idx: 0 })
        .insert(MapSelectionUI);

    commands.init_resource::<MapSelectionState>();
}

pub fn cleanup_systems(mut commands: Commands, qtui: Query<Entity, With<MapSelectionUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut map_selection_state: ResMut<MapSelectionState>,
    maps: Res<Maps>,
    mut next_state: ResMut<NextState<MapHubState>>,
    mut root_next_state: ResMut<NextState<root::State>>,
    mut ev_map_selected: EventWriter<MapSelectedEvent>,
) {
    let map_count = maps.maps.len();

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if map_selection_state.selected_map_idx == 0 {
            // If we are at "Go Back", wrap to the last map
            map_selection_state.selected_map_idx = map_count;
        } else {
            map_selection_state.selected_map_idx -= 1;
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        map_selection_state.selected_map_idx =
            (map_selection_state.selected_map_idx + 1) % (map_count + 1); // Wrap around, including "Go Back"
    } else if keyboard_input.just_pressed(KeyCode::Enter) {
        if map_selection_state.selected_map_idx < map_count {
            // Send the MapSelectedEvent only if a map is selected
            ev_map_selected.send(MapSelectedEvent {
                map_idx: map_selection_state.selected_map_idx,
            });

            // Transition to the difficulty selection screen
            next_state.set(MapHubState::DifficultySelection);
        } else {
            // If "Go Back" is selected, transition back to the main menu
            root_next_state.set(root::State::MainMenu);
            next_state.set(MapHubState::None);
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        // Transition back to the main menu
        root_next_state.set(root::State::MainMenu);
        next_state.set(MapHubState::None);
    }
}

pub fn update_item_colors(
    mut q_items: Query<(&MapSelectionItem, &Children)>,
    map_selection_state: Res<MapSelectionState>,
    mut q_text: Query<&mut Text>,
) {
    // Iterate through all map items
    for (map_item, children) in q_items.iter_mut() {
        // Get the text child of the button
        if let Ok(mut text) = q_text.get_mut(children[0]) {
            // Iterate through text sections
            for section in text.sections.iter_mut() {
                // Set the text color based on whether the item is selected
                let new_color = if map_item.map_idx == map_selection_state.selected_map_idx {
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
}

pub fn setup_ui(commands: &mut Commands, handles: &root::GameAssets, maps: &Maps) {
    const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;

    let main_color = Color::Rgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                //    align_self: AlignSelf::Center,
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
        .insert(MapSelectionUI)
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
                        "Map Hub - Select Map",
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
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(80.0),
                                justify_content: JustifyContent::SpaceEvenly,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            background_color: main_color.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            for (i, map) in maps.maps.iter().enumerate() {
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
                                        background_color: Color::NONE.into(),
                                        ..default()
                                    })
                                    .insert(MapSelectionItem { map_idx: i })
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            map.name.clone(),
                                            TextStyle {
                                                font: handles.fonts.londrina.w300_light.clone(),
                                                font_size: 32.0,
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
                            background_color: Color::NONE.into(),
                            ..default()
                        })
                        .insert(MapSelectionItem {
                            map_idx: maps.maps.len(),
                        }) // Set map_idx to map_count for "Go Back"
                        .with_children(|btn| {
                            btn.spawn(TextBundle::from_section(
                                "Go Back",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 38.0,
                                    color: colors::MENU_ITEM_COLOR_OFF,
                                },
                            ));
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
        });
    info!("MapHub - Map Selection menu loaded");
}
