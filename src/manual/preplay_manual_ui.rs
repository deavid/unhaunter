//! This module implements the UI and systems for the pre-play manual,
//! which is shown as a tutorial before starting a new game on certain difficulty levels.

use crate::{
    difficulty::CurrentDifficulty,
    game::level::LoadLevelEvent,
    manual::ManualPage,
    maphub::difficulty_selection::DifficultySelectionState,
    root::{self, GameAssets},
};
use bevy::prelude::*;
use enum_iterator::Sequence as _;

use super::{user_manual_ui::PageContent, utils::draw_page_content, ManualChapter};

#[derive(Component)]
pub struct ManualCamera;

#[derive(Component)]
pub struct PrePlayManualUI;

#[derive(Component, Clone)]
pub struct Input {
    pub keys: Vec<KeyCode>,
}

impl Input {
    pub fn from_keys(keys: impl IntoIterator<Item = KeyCode>) -> Self {
        Self {
            keys: keys.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct ManualEvent {
    pub action: ManualButtonAction,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualButtonAction {
    Previous,
    Continue,
}

#[allow(clippy::too_many_arguments)]
pub fn preplay_manual_system(
    mut current_page: ResMut<ManualPage>,
    mut ev_level: EventWriter<LoadLevelEvent>,
    difficulty: Res<CurrentDifficulty>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    maps: Res<root::Maps>,
    mut next_state: ResMut<NextState<root::State>>,
    mut evr_manual_button: EventReader<ManualEvent>,
) {
    for ev in evr_manual_button.read() {
        match ev.action {
            ManualButtonAction::Previous => {
                if *current_page == ManualPage::first().unwrap() {
                    next_state.set(root::State::MapHub);
                } else {
                    *current_page = current_page.previous().unwrap_or(*current_page);
                }
            }
            ManualButtonAction::Continue => {
                if difficulty.0.tutorial_chapter.is_some() {
                    if *current_page == ManualPage::last().unwrap() {
                        let map_filepath = maps.maps[difficulty_selection_state.selected_map_idx]
                            .path
                            .clone();
                        ev_level.send(LoadLevelEvent { map_filepath });
                        next_state.set(root::State::InGame);
                    } else {
                        *current_page = current_page.next().unwrap_or(*current_page);
                    }
                } else {
                    let map_filepath = maps.maps[difficulty_selection_state.selected_map_idx]
                        .path
                        .clone();
                    ev_level.send(LoadLevelEvent { map_filepath });
                    next_state.set(root::State::InGame);
                }
            }
        }
    }
}

/// Draws the pre-play manual UI, which guides the player through a tutorial.
pub fn draw_manual_ui(
    commands: &mut Commands,
    handles: Res<GameAssets>,
    current_page: &ManualPage,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            ..default()
        })
        .insert(PrePlayManualUI)
        .with_children(|parent| {
            // Page Content Container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        flex_basis: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(PageContent)
                .with_children(|content| draw_page_content(content, &handles, *current_page));

            // Navigation Buttons
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(5.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Percent(3.0)),
                        flex_grow: 0.0,
                        flex_basis: Val::Percent(5.0),

                        ..default()
                    },

                    ..default()
                })
                .with_children(|buttons| {
                    // Previous Button
                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Percent(30.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),

                                ..default()
                            },

                            background_color: Color::BLACK.with_alpha(0.2).into(),

                            ..default()
                        })
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "Previous",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ));
                        })
                        .insert(ManualButtonAction::Previous)
                        .insert(Input::from_keys([KeyCode::Escape])); // <-- Key binding

                    // Continue Button
                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Percent(30.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },

                            background_color: Color::BLACK.with_alpha(0.2).into(),
                            ..default()
                        })
                        .with_children(|button| {
                            button.spawn(TextBundle::from_section(
                                "Continue",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ));
                        })
                        .insert(ManualButtonAction::Continue)
                        .insert(Input::from_keys([
                            KeyCode::Space,
                            KeyCode::ArrowRight,
                            KeyCode::Enter,
                            KeyCode::KeyE,
                        ])); // <-- Key bindings
                });
        });
}

pub fn manual_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &Children,
            Option<&Input>,
            Option<&ManualButtonAction>,
        ),
        With<Button>,
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut manual_events: EventWriter<ManualEvent>,
) {
    for (interaction, _, maybe_input, maybe_action) in &mut interaction_query {
        // --- Mouse Click Handling ---
        if *interaction == Interaction::Pressed {
            if let Some(action) = maybe_action {
                // Directly use the ManualButtonAction component if present
                manual_events.send(ManualEvent { action: *action });
            }
        }

        // --- Keyboard Input Handling ---
        if let Some(input) = maybe_input {
            for key in &input.keys {
                if keyboard_input.just_pressed(*key) {
                    if let Some(action) = maybe_action {
                        manual_events.send(ManualEvent { action: *action });
                    }
                }
            }
        }
    }
}

pub fn setup_preplay_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    difficulty: Res<CurrentDifficulty>,
) {
    let initial_page = match difficulty.0.tutorial_chapter {
        Some(ManualChapter::Chapter1) => ManualPage::MissionBriefing, // First page of Chapter 1
        _ => ManualPage::MissionBriefing, // Default to the first page, for when calling from the main menu
    };

    commands.insert_resource(initial_page); // Update initial page
    commands
        .spawn(Camera2dBundle::default())
        .insert(ManualCamera);

    //Draw the PrePlay UI.
    draw_manual_ui(&mut commands, handles, &initial_page);
}

pub fn cleanup_preplay_ui(
    mut commands: Commands,
    q_manual_ui: Query<Entity, With<PrePlayManualUI>>,
    q_camera: Query<Entity, With<ManualCamera>>,
) {
    // Despawn the manual UI
    for entity in q_manual_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn the manual camera
    for entity in q_camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn start_preplay_manual_system(
    difficulty: Res<CurrentDifficulty>,
    mut next_game_state: ResMut<NextState<root::State>>,
) {
    // Check if a tutorial chapter is assigned for the current difficulty.
    if difficulty.0.tutorial_chapter.is_some() {
        next_game_state.set(root::State::PreplayManual);
    }
}

fn redraw_manual_ui_system(
    mut commands: Commands,
    current_page: Res<ManualPage>,
    q_manual_ui: Query<Entity, With<PrePlayManualUI>>,
    q_page_content: Query<Entity, With<PageContent>>,
    handles: Res<GameAssets>,
) {
    if !current_page.is_changed() {
        // Only redraw if the page has changed
        return;
    }

    // Get the ManualUI entity
    let Ok(_) = q_manual_ui.get_single() else {
        return;
    };

    // Get the PageContent entity
    let Ok(page_content_entity) = q_page_content.get_single() else {
        return;
    };

    // Despawn the existing page content
    commands.entity(page_content_entity).despawn_descendants();

    // Redraw the page content
    commands
        .entity(page_content_entity)
        .with_children(|parent| {
            draw_page_content(parent, &handles, *current_page);
        });
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::PreplayManual), setup_preplay_ui)
        .add_systems(OnExit(root::State::PreplayManual), cleanup_preplay_ui)
        .add_systems(
            Update,
            (
                manual_button_system,
                preplay_manual_system,
                redraw_manual_ui_system,
            )
                .chain()
                .run_if(in_state(root::State::PreplayManual)),
        )
        .add_event::<ManualEvent>();
}
