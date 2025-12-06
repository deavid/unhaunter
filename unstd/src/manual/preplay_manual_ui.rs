//! This module implements the UI and systems for the pre-play manual,
//! which is shown as a tutorial before starting a new game on certain difficulty levels.
use super::{CurrentManualPage, Manual};
use bevy::prelude::*;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::loadlevel::LoadLevelEvent;
use uncore::platform::plt::FONT_SCALE;
use uncore::resources::difficulty_state::DifficultySelectionState;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::types::root::game_assets::GameAssets;

use super::draw_manual_page;

#[derive(Component)]
pub struct ManualCamera;

#[derive(Component)]
pub struct PageContent;

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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreplayManualNavigationAction {
    Continue,
    Previous,
}

#[derive(Debug, Clone, Copy, Event)]
pub struct PreplayManualNavigationEvent {
    pub action: PreplayManualNavigationAction,
}

// System for handling user interaction and page navigation within the pre-play manual.
pub fn preplay_manual_system(
    mut evr_manual_button: EventReader<PreplayManualNavigationEvent>,
    mut current_manual_page: ResMut<CurrentManualPage>,
    difficulty: Res<CurrentDifficulty>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    maps: Res<Maps>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_load_level: EventWriter<LoadLevelEvent>,
    manual: Res<Manual>,
) {
    for ev in evr_manual_button.read() {
        match ev.action {
            PreplayManualNavigationAction::Previous => {
                if current_manual_page.1 > 0 {
                    current_manual_page.1 -= 1; // Go to previous page
                } else {
                    // Return to map/difficulty selection
                    next_state.set(AppState::MapHub);
                }
            }

            PreplayManualNavigationAction::Continue => {
                if let Some(Some(chapter)) = difficulty
                    .0
                    .tutorial_chapter
                    .map(|c| manual.chapters.get(c.index()))
                {
                    let current_chapter_size = chapter.pages.len();

                    if current_manual_page.1 + 1 < current_chapter_size {
                        current_manual_page.1 += 1;
                    } else {
                        // Last page, start game
                        let map_filepath = maps.maps[difficulty_selection_state.selected_map_idx]
                            .path
                            .clone();
                        ev_load_level.write(LoadLevelEvent { map_filepath });
                        next_state.set(AppState::Loading);
                    }
                } else {
                    // No tutorial chapter, start game immediately.
                    let map_filepath = maps.maps[difficulty_selection_state.selected_map_idx]
                        .path
                        .clone();

                    ev_load_level.write(LoadLevelEvent { map_filepath });
                    next_state.set(AppState::Loading);
                }
            }
        }
    }
}

fn manual_button_system(
    mut interaction_query: Query<
        (
            Ref<Interaction>,
            Option<&Input>,
            Option<&PreplayManualNavigationAction>,
        ),
        With<Button>,
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut manual_events: EventWriter<PreplayManualNavigationEvent>,
) {
    for (interaction, maybe_input, maybe_action) in &mut interaction_query {
        if interaction.is_changed()
            && *interaction == Interaction::Pressed
            && let Some(action) = maybe_action
        {
            manual_events.write(PreplayManualNavigationEvent { action: *action });
        }

        // --- Keyboard input handling ---
        if let Some(input) = maybe_input {
            for key in &input.keys {
                if keyboard_input.just_pressed(*key)
                    && let Some(action) = maybe_action
                {
                    manual_events.write(PreplayManualNavigationEvent { action: *action });
                }
            }
        }
    }
}

/// Draws the pre-play manual UI, which guides the player through a tutorial.
pub fn draw_manual_ui(commands: &mut Commands, handles: Res<GameAssets>) {
    let button_text_style = TextFont {
        font: handles.fonts.londrina.w300_light.clone(),
        font_size: 30.0 * FONT_SCALE,
        ..default()
    };

    let prev_button = |button: &mut ChildSpawnerCommands| {
        button
            .spawn(Text::new("Previous"))
            .insert(button_text_style.clone())
            .insert(TextColor(Color::WHITE));
    };
    let continue_button = |button: &mut ChildSpawnerCommands| {
        button
            .spawn(Text::new("Continue"))
            .insert(button_text_style.clone())
            .insert(TextColor(Color::WHITE));
    };
    let nav_buttons = |buttons: &mut ChildSpawnerCommands| {
        // Previous button
        buttons
            .spawn(Button)
            .insert(Node {
                width: Val::Percent(30.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            })
            .insert(BackgroundColor(Color::BLACK.with_alpha(0.2)))
            .with_children(prev_button)
            .insert(PreplayManualNavigationAction::Previous)
            .insert(Input::from_keys([KeyCode::Escape, KeyCode::ArrowLeft]));

        // Continue Button
        buttons
            .spawn(Button)
            .insert(Node {
                width: Val::Percent(30.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            })
            .insert(BackgroundColor(Color::BLACK.with_alpha(0.2)))
            .with_children(continue_button)
            .insert(PreplayManualNavigationAction::Continue)
            .insert(Input::from_keys([
                KeyCode::Space,
                KeyCode::ArrowRight,
                KeyCode::Enter,
                KeyCode::KeyE,
            ]));
    };
    let page_content = |parent: &mut ChildSpawnerCommands| {
        // Page Content Container - Now Empty, content will be added later
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: Val::Percent(100.0),
                ..default()
            })
            .insert(PageContent);

        // Navigation Buttons
        parent
            .spawn(Node {
                width: Val::Percent(90.0),
                height: Val::Percent(5.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Percent(3.0)),
                flex_grow: 0.0,
                flex_basis: Val::Percent(5.0),
                ..default()
            })
            .with_children(nav_buttons);
    };

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(2.0)),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(PrePlayManualUI)
        .with_children(|parent| {
            // Add menu background first
            parent
                .spawn(ImageNode {
                    image: handles.images.menu_background_low_contrast.clone(),
                    ..default()
                })
                .insert(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                })
                .insert(ZIndex(-10));

            // Then add the content on top
            page_content(parent);
        });
}

pub fn setup_preplay_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    difficulty: Res<CurrentDifficulty>,
) {
    commands.insert_resource(CurrentManualPage(
        difficulty
            .0
            .tutorial_chapter
            .as_ref()
            .map(|x| x.index())
            .unwrap_or_default(),
        0,
    ));
    commands.spawn(Camera2d).insert(ManualCamera);

    draw_manual_ui(&mut commands, handles);
}

pub fn cleanup_preplay_ui(
    mut commands: Commands,
    q_manual_ui: Query<Entity, With<PrePlayManualUI>>,
    q_camera: Query<Entity, With<ManualCamera>>,
) {
    for entity in q_manual_ui.iter() {
        commands.entity(entity).despawn();
    }

    for entity in q_camera.iter() {
        commands.entity(entity).despawn();
    }
}

fn redraw_manual_ui_system(
    mut commands: Commands,
    current_manual_page: Res<CurrentManualPage>,
    q_manual_ui: Query<Entity, With<PrePlayManualUI>>,
    q_page_content: Query<Entity, With<PageContent>>,
    handles: Res<GameAssets>,
    manuals: Res<Manual>,
) {
    // Get the ManualUI entity
    let Ok(_) = q_manual_ui.single() else {
        return;
    };

    // Get the PageContent entity
    let Ok(page_content_entity) = q_page_content.single() else {
        return;
    };

    // Despawn the existing page content
    commands
        .entity(page_content_entity)
        .despawn_related::<Children>();

    // Redraw the page content
    commands
        .entity(page_content_entity)
        .with_children(|parent| {
            draw_manual_page(parent, &handles, &manuals, &current_manual_page);
        });
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::PreplayManual), setup_preplay_ui)
        .add_systems(OnExit(AppState::PreplayManual), cleanup_preplay_ui)
        .add_systems(
            Update,
            (
                manual_button_system,
                preplay_manual_system,
                redraw_manual_ui_system,
            )
                .chain()
                .run_if(in_state(AppState::PreplayManual)),
        )
        .add_event::<PreplayManualNavigationEvent>(); //Add event
}
