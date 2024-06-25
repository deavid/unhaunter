pub mod chapter1;
pub mod index;

use bevy::prelude::*;

use enum_iterator::Sequence;

use crate::{
    difficulty::CurrentDifficulty,
    root::{self, GameAssets},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Resource)]
pub enum ManualPage {
    Introduction,
    BasicControls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManualPageRange {
    pub entry_page: ManualPage,
    pub exit_page: ManualPage,
}

impl ManualPageRange {
    pub fn new(entry_page: ManualPage, exit_page: ManualPage) -> Self {
        // Ensure entry_page comes before or is the same as exit_page in the enum sequence
        let all_pages: Vec<ManualPage> = enum_iterator::all::<ManualPage>().collect();
        let entry_idx = all_pages
            .iter()
            .enumerate()
            .find_map(|(n, v)| if *v == entry_page { Some(n) } else { None })
            .unwrap();
        let exit_idx = all_pages
            .iter()
            .enumerate()
            .find_map(|(n, v)| if *v == exit_page { Some(n) } else { None })
            .unwrap();

        if entry_idx > exit_idx {
            panic!("Invalid ManualPageRange: entry_page must come before or be the same as exit_page in the enum sequence.");
        }

        Self {
            entry_page,
            exit_page,
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn manual_system(
    mut current_page: ResMut<ManualPage>,
    difficulty: Res<CurrentDifficulty>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<root::State>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    text_query: Query<&Text>, // Query for Text components
) {
    // Store the manual UI entity ID
    // let mut manual_ui_entity = None;

    // Handle button clicks
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok(text) = text_query.get(*child) {
                    if text.sections[0].value == "Previous" {
                        *current_page = current_page.previous().unwrap_or(*current_page);
                    // Use previous() from Sequence
                    } else if text.sections[0].value == "Next" {
                        *current_page = current_page.next().unwrap_or(*current_page);
                    // Use next() from Sequence
                    } else if text.sections[0].value == "Close" {
                        // Transition back to the appropriate state
                        if difficulty.0.manual_pages.entry_page == *current_page {
                            next_state.set(root::State::MainMenu);
                        } else {
                            game_next_state.set(root::GameState::None);
                        }
                    }
                }
            }
        }
    }

    // Handle left/right arrow keys
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        *current_page = current_page.previous().unwrap_or(*current_page);
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        *current_page = current_page.next().unwrap_or(*current_page);
    }

    // FIXME: This is wrong. It seems that it attempts a redraw but this is incorrect.
    // Redraw the UI with the updated page
    // if let Some(entity) = manual_ui_entity {
    //     commands.entity(entity).despawn_descendants();
    // }
    // manual_ui_entity = Some(draw_manual_ui(&mut commands, &handles, *current_page));
    // Pass &mut commands
}

#[derive(Component)]
pub struct ManualCamera;

pub fn setup(mut commands: Commands, handles: Res<GameAssets>, difficulty: Res<CurrentDifficulty>) {
    // Set the initial page based on the difficulty
    let initial_page = difficulty.0.manual_pages.entry_page;
    commands.insert_resource(initial_page);

    // Spawn the 2D camera for the manual UI
    commands
        .spawn(Camera2dBundle::default())
        .insert(ManualCamera);

    // Draw the manual UI
    index::draw_manual_ui(&mut commands, handles, &initial_page);
}

pub fn cleanup(
    mut commands: Commands,
    q_manual_ui: Query<Entity, With<index::ManualUI>>,
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
        .add_systems(Update, manual_system.run_if(in_state(root::State::Manual)))
        .add_systems(
            Update,
            manual_system.run_if(in_state(root::GameState::Manual)),
        );
}
