//! This module implements the UI and systems for the pre-play manual,
//! which is shown as a tutorial before starting a new game on certain difficulty levels.

use crate::{
    difficulty::CurrentDifficulty,
    game::level::LoadLevelEvent,
    manual::ManualPage,
    maphub::difficulty_selection::DifficultySelectionState, // Import for map filepath
    root::{self, GameAssets},
};
use bevy::prelude::*;
use enum_iterator::Sequence as _;

use super::{user_manual_ui::PageContent, utils::draw_page_content, ManualChapter};

#[derive(Component)]
pub struct ManualCamera;

#[derive(Component)]
pub struct PrePlayManualUI;

/// System for handling user interaction and page navigation within the pre-play manual.
#[allow(clippy::too_many_arguments)]
pub fn preplay_manual_system(
    mut current_page: ResMut<ManualPage>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<root::State>>,
    mut text_query: Query<&mut Text>,
    mut ev_load_level: EventWriter<LoadLevelEvent>,
    difficulty: Res<CurrentDifficulty>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    maps: Res<root::Maps>,
) {
    // --- Interaction Handling ---
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    match text.sections[0].value.as_str() {
                        "Previous" => {
                            if *current_page == ManualPage::first().unwrap() {
                                // If on the first page -> quit and go back to difficulty menu
                                next_state.set(root::State::MapHub);
                            } else {
                                // Otherwise go back one page
                                *current_page = current_page.previous().unwrap_or(*current_page);
                            }
                        }

                        "Continue" => {
                            // Renamed "Next" to "Continue"
                            if difficulty.0.tutorial_chapter.is_some() {
                                if *current_page == ManualPage::last().unwrap() {
                                    // Last page of tutorial, start game
                                    let map_filepath = maps.maps
                                        [difficulty_selection_state.selected_map_idx]
                                        .path
                                        .clone();
                                    ev_load_level.send(LoadLevelEvent { map_filepath });
                                    next_state.set(root::State::InGame);
                                } else {
                                    // Not last page, advance to the next page
                                    *current_page = current_page.next().unwrap_or(*current_page);
                                }
                            } else {
                                // No tutorial chapter, start game directly
                                let map_filepath = maps.maps
                                    [difficulty_selection_state.selected_map_idx]
                                    .path
                                    .clone();
                                ev_load_level.send(LoadLevelEvent { map_filepath });
                                next_state.set(root::State::InGame);
                            }
                        }
                        &_ => {}
                    }
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
                        });

                    // Next Button
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
                                "Next",
                                TextStyle {
                                    font: handles.fonts.londrina.w300_light.clone(),
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
        });
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

    commands.insert_resource(initial_page); //Update initial page
    commands
        .spawn(Camera2dBundle::default()) //Respawning the camera just in case - not ideal
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
            (preplay_manual_system, redraw_manual_ui_system)
                .chain()
                .run_if(in_state(root::State::PreplayManual)),
        );
}
