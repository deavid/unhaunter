pub mod chapter1;
pub mod index;

use bevy::prelude::*;

use enum_iterator::Sequence;
use index::{ManualUI, PageContent};

use crate::{
    difficulty::CurrentDifficulty,
    root::{self, GameAssets},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Resource, Default)]
pub enum ManualPage {
    #[default]
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

fn manual_system(
    mut current_page: ResMut<ManualPage>,
    difficulty: Res<CurrentDifficulty>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<root::State>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    text_query: Query<&Text>, // Query for Text components
    mut button_query: Query<(&Children, &mut Visibility), With<Button>>,
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
                        info!("Current page: {:?}", *current_page);
                    // Use previous() from Sequence
                    } else if text.sections[0].value == "Next" {
                        *current_page = current_page.next().unwrap_or(*current_page);
                        info!("Current page: {:?}", *current_page);
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

fn redraw_manual_ui_system(
    mut commands: Commands,
    current_page: Res<ManualPage>,
    q_manual_ui: Query<Entity, With<ManualUI>>,
    q_page_content: Query<Entity, With<PageContent>>,
    handles: Res<GameAssets>,
) {
    if !current_page.is_changed() {
        return; // Only redraw if the page has changed
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
            index::draw_manual_page(parent, &handles, *current_page);
        });
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
        .add_systems(
            Update,
            manual_system
                .run_if(in_state(root::State::Manual).or_else(in_state(root::GameState::Manual))),
        )
        .add_systems(
            Update,
            redraw_manual_ui_system
                .run_if(in_state(root::State::Manual).or_else(in_state(root::GameState::Manual))) // Add run_if condition here
                .after(manual_system),
        )
        .insert_resource(ManualPage::default());
}
