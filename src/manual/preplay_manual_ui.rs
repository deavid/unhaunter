//! This module implements the UI and systems for the pre-play manual,
//! which is shown as a tutorial before starting a new game on certain difficulty levels.

use crate::{
    difficulty::CurrentDifficulty,
    manual::{chapter1, ManualPage},
    root::{self, GameAssets},
};
use bevy::prelude::*;
use enum_iterator::Sequence as _;

use super::user_manual_ui::PageContent;

/// Marker component for the pre-play manual UI.
#[derive(Component)]
pub struct PrePlayManualUI;

/// Component to store a timer for automatic page advancement in the pre-play manual.
#[derive(Component)]
pub struct PrePlayManualTimer(Timer);

/// System for handling user interaction and page navigation within the pre-play manual.
#[allow(clippy::too_many_arguments)]
pub fn preplay_manual_system(
    mut current_page: ResMut<ManualPage>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<root::State>>,
    mut text_query: Query<&mut Text>,
    mut button_query: Query<(&Children, &mut Visibility), With<Button>>,
    time: Res<Time>,
    mut timers: Query<&mut PrePlayManualTimer>,
    difficulty: Res<CurrentDifficulty>,
) {
    let mut timer = timers.get_single_mut().unwrap(); // Safe because we check for emptiness earlier

    // Tick the timer and handle automatic page advancement
    timer.0.tick(time.delta());
    if timer.0.finished() && *current_page != ManualPage::last().unwrap() {
        *current_page = current_page.next().unwrap_or(*current_page);
        timer.0.reset();
    }

    // Update button visibility based on timer and current page
    for (children, mut visibility) in &mut button_query {
        for &child in children {
            if let Ok(text) = text_query.get(child) {
                match text.sections[0].value.as_str() {
                    "Previous" => {
                        *visibility = Visibility::Visible; // Always visible
                    }
                    "Next" => {
                        *visibility = if *current_page == ManualPage::last().unwrap() {
                            Visibility::Hidden // Hide on the last page
                        } else {
                            Visibility::Visible // Visible otherwise
                        };
                    }
                    _ => {} // Keep other buttons visible
                }
            }
        }
    }

    // --- Interaction Handling ---
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            for &child in children.iter() {
                if let Ok(text) = text_query.get_mut(child) {
                    match text.sections[0].value.as_str() {
                        "Previous" => {
                            if *current_page == ManualPage::first().unwrap()
                                && difficulty.0.tutorial_chapter.is_some()
                            {
                                // If on the first page of tutorial -> quit and go back to difficulty menu
                                next_state.set(root::State::MapHub);
                            } else {
                                // Otherwise go back one page
                                *current_page = current_page.previous().unwrap_or(*current_page);

                                // Pause and Reset Timer if available
                            }
                        }

                        "Next" => {
                            if *current_page == ManualPage::last().unwrap()
                                && difficulty.0.tutorial_chapter.is_some()
                            {
                                // If last page of tutorial go into the game.
                                next_state.set(root::State::InGame); // Set to the playing game mode
                            } else {
                                // Otherwise go next page
                                *current_page = current_page.next().unwrap_or(*current_page);

                                // Pause and Reset Timer if available
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
                .with_children(|content| draw_manual_page(content, &handles, *current_page));

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

pub fn draw_manual_page(parent: &mut ChildBuilder, handles: &GameAssets, current_page: ManualPage) {
    match current_page {
        ManualPage::EMFAndThermometer => {
            chapter1::p01_mission_briefing::draw_mission_briefing_page(parent, handles)
        }
        ManualPage::EssentialControls => {
            chapter1::p02_essential_controls::draw_essential_controls_page(parent, handles)
        }
        _ => {} // Add more pages as needed
    }
}