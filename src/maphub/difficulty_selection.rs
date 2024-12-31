use super::MapHubState;
use crate::colors;
use crate::difficulty::{CurrentDifficulty, Difficulty};
use crate::game::level::LoadLevelEvent;
use crate::manual::preplay_manual_ui::start_preplay_manual_system;
use crate::platform::plt::UI_SCALE;
use crate::root;
use bevy::prelude::*;

// Add MapSelectedEvent
use crate::maphub::map_selection::MapSelectedEvent;

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
    // Add this to store the selected map index
    pub selected_map_idx: usize,
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
        // Register the new event
        .add_event::<DifficultyConfirmedEvent>();
}

pub fn setup_systems(
    mut commands: Commands,
    // Access MapSelectedEvent
    mut ev_map_selected: EventReader<MapSelectedEvent>,
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
    mut next_hub_state: ResMut<NextState<MapHubState>>,
    mut difficulty: ResMut<CurrentDifficulty>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    maps: Res<root::Maps>,
    ev_load_level: EventWriter<LoadLevelEvent>,
    next_state: ResMut<NextState<root::State>>,
) {
    if ev_difficulty_confirmed.read().next().is_none() {
        return;
    }

    difficulty.0 = difficulty_selection_state
        .selected_difficulty
        .create_difficulty_struct();

    start_preplay_manual_system(
        difficulty.into(),
        next_state,
        difficulty_selection_state,
        maps,
        ev_load_level,
    );

    next_hub_state.set(MapHubState::None);
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
    mut q_textcolor: Query<&mut TextColor>,
    mut q_text: Query<&mut Text>,
    q_desc: Query<Entity, With<DifficultyDescriptionUI>>,
) {
    if !difficulty_selection_state.is_changed() {
        return;
    }
    let sel_difficulty = difficulty_selection_state.selected_difficulty;
    for (difficulty_item, children) in q_items.iter_mut() {
        if let Ok(mut textcolor) = q_textcolor.get_mut(children[0]) {
            let new_color = if difficulty_item.difficulty == sel_difficulty {
                colors::MENU_ITEM_COLOR_ON
            } else {
                colors::MENU_ITEM_COLOR_OFF
            };
            if new_color != textcolor.0 {
                textcolor.0 = new_color;
            }
        }
    }
    let dif = sel_difficulty.create_difficulty_struct();
    for entity in q_desc.iter() {
        if let Ok(mut text) = q_text.get_mut(entity) {
            let name = &dif.difficulty_name;
            let description = &dif.difficulty_description;
            let score_mult = dif.difficulty_score_multiplier;
            let new_text = [
                format!("Difficulty <{name}>: {description}"),
                format!("Score Bonus: {score_mult:.2}x"),
            ];
            text.0 = new_text.join("\n");
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
        .insert(BackgroundColor(main_color.into()))
        .insert(DifficultySelectionUI)
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    min_width: Val::Px(0.0),
                    min_height: Val::Px(64.0),
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
                        .spawn(Text::new("Map Hub - Select Difficulty"))
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

                    // Difficulty buttons in a 3-column grid
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(80.0),
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            display: Display::Grid,
                            // 4 equal columns
                            grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                            grid_auto_rows: GridTrack::auto(),
                            row_gap: Val::Px(10.0 * UI_SCALE),
                            column_gap: Val::Px(20.0 * UI_SCALE),
                            ..default()
                        })
                        .insert(BackgroundColor(main_color.into()))
                        .with_children(|parent| {
                            for difficulty in Difficulty::all() {
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
                                    .insert(DifficultySelectionItem { difficulty })
                                    .with_children(|btn| {
                                        btn.spawn(Text::new(difficulty.difficulty_name()))
                                            .insert(TextFont {
                                                font: handles.fonts.londrina.w300_light.clone(),
                                                font_size: 28.0 * UI_SCALE,
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
                });
            parent
                .spawn(Text::new("Difficulty <>: Description"))
                .insert(TextFont {
                    font: handles.fonts.titillium.w300_light.clone(),
                    font_size: 26.0 * UI_SCALE,
                    font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                })
                .insert(TextColor(colors::MENU_ITEM_COLOR_OFF))
                .insert(Node {
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
                })
                .insert(DifficultyDescriptionUI);
        });
    info!("MapHub - Difficulty menu loaded");
}
