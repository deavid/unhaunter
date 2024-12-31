use crate::colors;
use crate::platform::plt::UI_SCALE;
use crate::root::Maps;
use crate::{maphub::MapHubState, root};
use bevy::prelude::*;

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
        // Wrap around, including "Go Back"
        map_selection_state.selected_map_idx =
            (map_selection_state.selected_map_idx + 1) % (map_count + 1);
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
    mut q_textcolor: Query<&mut TextColor>,
) {
    // Iterate through all map items
    for (map_item, children) in q_items.iter_mut() {
        // Get the text child of the button
        if let Ok(mut textcolor) = q_textcolor.get_mut(children[0]) {
            // Set the text color based on whether the item is selected
            let new_color = if map_item.map_idx == map_selection_state.selected_map_idx {
                colors::MENU_ITEM_COLOR_ON
            } else {
                colors::MENU_ITEM_COLOR_OFF
            };
            if new_color != textcolor.0 {
                textcolor.0 = new_color;
            }
        }
    }
}

pub fn setup_ui(commands: &mut Commands, handles: &root::GameAssets, maps: &Maps) {
    const MARGIN_PERCENT: f32 = 0.5 * UI_SCALE;
    let main_color = Color::Srgba(Srgba {
        red: 0.2,
        green: 0.2,
        blue: 0.2,
        alpha: 0.05,
    });
    commands
        .spawn(Node {
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
            flex_grow: 1.0,
            ..default()
        })
        .insert(MapSelectionUI)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(64.0 * UI_SCALE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|parent| {
                    // logo
                    parent
                        .spawn(ImageNode {
                            image: handles.images.title.clone().into(),
                            ..default()
                        })
                        .insert(Node {
                            aspect_ratio: Some(130.0 / 17.0),
                            width: Val::Percent(80.0),
                            height: Val::Auto,
                            max_width: Val::Percent(80.0),
                            max_height: Val::Percent(100.0),
                            flex_shrink: 1.0,
                            ..default()
                        });
                });
            parent.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(20.0),
                ..default()
            });
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(60.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .insert(BackgroundColor(main_color.into()))
                .with_children(|parent| {
                    // text
                    parent
                        .spawn(Text::new("Map Hub - Select Map"))
                        .insert(TextFont {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 38.0 * UI_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::WHITE));
                    parent.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(10.0),
                        ..default()
                    });
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(80.0),
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            display: Display::Grid,
                            // 4 equal columns
                            grid_template_columns: vec![
                                GridTrack::flex(1.0),
                                GridTrack::flex(1.0),
                                GridTrack::flex(1.0),
                                GridTrack::flex(1.0),
                            ],
                            grid_auto_rows: GridTrack::auto(),
                            row_gap: Val::Px(10.0 * UI_SCALE),
                            column_gap: Val::Px(20.0 * UI_SCALE),
                            ..default()
                        })
                        .insert(BackgroundColor(main_color.into()))
                        .with_children(|parent| {
                            for (i, map) in maps.maps.iter().enumerate() {
                                parent
                                    .spawn(Button)
                                    .insert(Node {
                                        min_height: Val::Px(30.0 * UI_SCALE),
                                        border: UiRect::all(Val::Px(0.9 * UI_SCALE)),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                                        ..default()
                                    })
                                    .insert(BackgroundColor(Color::NONE.into()))
                                    .insert(MapSelectionItem { map_idx: i })
                                    .with_children(|btn| {
                                        btn.spawn(Text::new(map.name.clone()))
                                            .insert(TextFont {
                                                font: handles.fonts.londrina.w300_light.clone(),
                                                font_size: 32.0 * UI_SCALE,
                                                font_smoothing:
                                                    bevy::text::FontSmoothing::AntiAliased,
                                            })
                                            .insert(TextColor(colors::MENU_ITEM_COLOR_OFF));
                                    });
                            }
                        });
                    parent.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(10.0),
                        ..default()
                    });
                    parent
                        .spawn(Button)
                        .insert(Node {
                            min_height: Val::Px(30.0 * UI_SCALE),
                            border: UiRect::all(Val::Px(0.9 * UI_SCALE)),
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Percent(MARGIN_PERCENT)),
                            ..default()
                        })
                        .insert(BackgroundColor(Color::NONE.into()))
                        .insert(MapSelectionItem {
                            map_idx: maps.maps.len(),
                            // Set map_idx to map_count for "Go Back"
                        })
                        .with_children(|btn| {
                            btn.spawn(Text::new("Go Back"))
                                .insert(TextFont {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 38.0 * UI_SCALE,
                                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                                })
                                .insert(TextColor(colors::MENU_ITEM_COLOR_OFF));
                        });
                });
            parent.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(20.0),
                ..default()
            });
        });
    info!("MapHub - Map Selection menu loaded");
}
