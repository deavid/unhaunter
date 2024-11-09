use super::{utils::draw_page_content, ManualPage};
use crate::{
    difficulty::CurrentDifficulty,
    root::{self, GameAssets},
};
use bevy::prelude::*;
use enum_iterator::Sequence as _;

#[derive(Component)]
pub struct UserManualUI;

#[derive(Component)]
pub struct PageContent;

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
        .insert(UserManualUI)
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
                .with_children(|content| {
                    draw_page_content(content, &handles, *current_page);
                });

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

                    // Close Button
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
                                "Close",
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

pub fn user_manual_system(
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
    draw_manual_ui(&mut commands, handles, &initial_page);
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
    q_manual_ui: Query<Entity, With<UserManualUI>>,
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
    app.add_systems(OnEnter(root::State::UserManual), setup)
        .add_systems(OnExit(root::State::UserManual), cleanup)
        .add_systems(
            Update,
            user_manual_system.run_if(in_state(root::State::UserManual)),
        )
        .add_systems(
            Update,
            redraw_manual_ui_system
                // Add run_if condition here
                .run_if(in_state(root::State::UserManual))
                .after(user_manual_system),
        )
        .insert_resource(ManualPage::default());
}
