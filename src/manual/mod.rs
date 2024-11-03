pub mod chapter1;
pub mod preplay_manual_ui;
pub mod user_manual_ui;
pub mod utils;

use crate::{
    difficulty::CurrentDifficulty,
    root::{self, GameAssets},
};
use bevy::prelude::*;
use enum_iterator::Sequence;
pub use preplay_manual_ui::preplay_manual_system;
use user_manual_ui::{PageContent, UserManualUI};
use utils::draw_page_content;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualChapter {
    Chapter1,
    // Chapter2, // Add more chapters as needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Resource, Default)]
pub enum ManualPage {
    #[default]
    MissionBriefing,
    EssentialControls,
    EMFAndThermometer,
    TruckJournal,
    ExpellingGhost,
}

fn user_manual_system(
    mut current_page: ResMut<ManualPage>,
    // difficulty: Res<CurrentDifficulty>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<root::State>>,
    // mut game_next_state: ResMut<NextState<root::GameState>>,
    // Query for Text components
    text_query: Query<&Text>,
    mut button_query: Query<(&Children, &mut Visibility), With<Button>>,
) {
    // Store the manual UI entity ID let mut manual_ui_entity = None; Handle button
    // clicks
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok(text) = text_query.get(*child) {
                    if text.sections[0].value == "Previous" {
                        *current_page = current_page.previous().unwrap_or(*current_page);
                        info!("Current page: {:?}", *current_page);
                        // Use previous() from Sequence
                    } else if text.sections[0].value == "Next" {
                        *current_page = current_page.next().unwrap_or(*current_page);
                        info!("Current page: {:?}", *current_page);
                        // Use next() from Sequence
                    } else if text.sections[0].value == "Close" {
                        // Transition back to the appropriate state
                        next_state.set(root::State::MainMenu);
                        // FIXME: The following code is wrong. We should store where to go back in some place that is deterministic.
                        // ...    Also this needs to happen for Keyboard [ESC] too.
                        // if difficulty.0.manual_pages.entry_page == *current_page {
                        //     next_state.set(root::State::MainMenu);
                        // } else {
                        //     game_next_state.set(root::GameState::None);
                        // }
                    }
                }
            }
        }
    }

    // Handle left/right arrow keys and ESC key
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        *current_page = current_page.previous().unwrap_or(*current_page);
        info!("Current page: {:?}", *current_page);
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        *current_page = current_page.next().unwrap_or(*current_page);
        info!("Current page: {:?}", *current_page);
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        // Transition back to the main menu when ESC is pressed
        next_state.set(root::State::MainMenu);
    }

    // Update button visibility based on current page
    for (children, mut visibility) in &mut button_query {
        for child in children.iter() {
            if let Ok(text) = text_query.get(*child) {
                let is_first = text.sections[0].value == "Previous"
                    && *current_page == ManualPage::first().unwrap();
                let is_last = text.sections[0].value == "Next"
                    && *current_page == ManualPage::last().unwrap();
                *visibility = if is_first || is_last {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}
#[derive(Component)]
pub struct ManualCamera;

pub fn setup(
    mut commands: Commands,
    handles: Res<GameAssets>,
    _difficulty: Res<CurrentDifficulty>,
) {
    // Set the initial page based on the difficulty
    let initial_page = ManualPage::default();
    commands.insert_resource(initial_page);

    // Spawn the 2D camera for the manual UI
    commands
        .spawn(Camera2dBundle::default())
        .insert(ManualCamera);

    // Draw the manual UI
    user_manual_ui::draw_manual_ui(&mut commands, handles, &initial_page);
}

fn redraw_manual_ui_system(
    mut commands: Commands,
    current_page: Res<ManualPage>,
    q_manual_ui: Query<Entity, With<UserManualUI>>,
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

pub fn cleanup(
    mut commands: Commands,
    q_manual_ui: Query<Entity, With<user_manual_ui::UserManualUI>>,
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

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::State::Manual), setup)
        .add_systems(OnExit(root::State::Manual), cleanup)
        .add_systems(OnEnter(root::GameState::Manual), setup)
        .add_systems(OnExit(root::GameState::Manual), cleanup)
        .add_systems(
            Update,
            user_manual_system.run_if(in_state(root::State::Manual)),
        )
        .add_systems(
            Update,
            // FIXME: This Run-if is wrong, needs fixing.
            preplay_manual_system.run_if(in_state(root::GameState::Manual)),
        )
        .add_systems(
            Update,
            redraw_manual_ui_system
                // Add run_if condition here
                .run_if(in_state(root::State::Manual))
                .after(user_manual_system),
        )
        .insert_resource(ManualPage::default());
}
