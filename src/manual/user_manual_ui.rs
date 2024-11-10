use super::{
    draw_manual_page, utils::draw_page_content_obsolete, CurrentManualPage, Manual,
    ManualPageObsolete,
};
use crate::{
    difficulty::CurrentDifficulty,
    root::{self, GameAssets},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct ManualCamera;

#[derive(Component)]
pub struct UserManualUI;

#[derive(Component)]
pub struct PageContent;

pub fn draw_manual_ui(
    commands: &mut Commands,
    handles: Res<GameAssets>,
    current_page: &ManualPageObsolete,
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
                    draw_page_content_obsolete(content, &handles, *current_page);
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
    mut current_manual_page: ResMut<CurrentManualPage>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<root::State>>,
    text_query: Query<&Text>,
    mut button_query: Query<(&Children, &mut Visibility), With<Button>>,
    manuals: Res<Manual>,
) {
    for (interaction, children) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok(text) = text_query.get(*child) {
                    match text.sections[0].value.as_str() {
                        "Previous" => {
                            if current_manual_page.1 > 0 {
                                // Go to the previous page in the chapter
                                current_manual_page.1 -= 1;
                            } else {
                                warn!(
                                    "We're at the first page of chapter {}",
                                    current_manual_page.0
                                );
                            }
                        }
                        "Next" => {
                            let current_chapter_size =
                                manuals.chapters[current_manual_page.0].pages.len();
                            if current_manual_page.1 + 1 < current_chapter_size {
                                // Go to the next page of the chapter
                                current_manual_page.1 += 1;
                            } else {
                                warn!(
                                    "We're at the last page of chapter {}",
                                    current_manual_page.0
                                );
                            }
                        }
                        "Close" => next_state.set(root::State::MainMenu),
                        _ => (),
                    }
                }
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        if current_manual_page.1 > 0 {
            current_manual_page.1 -= 1; // Go to the previous page in the chapter
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        let current_chapter_size = manuals.chapters[current_manual_page.0].pages.len();
        if current_manual_page.1 + 1 < current_chapter_size {
            current_manual_page.1 += 1; // Go to the next page of the chapter
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(root::State::MainMenu);
    }

    for (children, mut visibility) in &mut button_query {
        for child in children.iter() {
            if let Ok(text) = text_query.get(*child) {
                let is_first = text.sections[0].value == "Previous" && current_manual_page.1 == 0;
                let current_chapter_size = manuals.chapters[current_manual_page.0].pages.len();

                let is_last = text.sections[0].value == "Next"
                    && current_manual_page.1 + 1 == current_chapter_size;

                *visibility = if is_first || is_last {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}

pub fn setup(
    mut commands: Commands,
    handles: Res<GameAssets>,
    _difficulty: Res<CurrentDifficulty>,
) {
    // Set the initial page based on the difficulty
    let initial_page = ManualPageObsolete::default();
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
    current_manual_page: Res<CurrentManualPage>,
    q_manual_ui: Query<Entity, With<UserManualUI>>,
    q_page_content: Query<Entity, With<PageContent>>,
    handles: Res<GameAssets>,
    manuals: Res<Manual>,
) {
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

    // Redraw the page content, changed "draw_page_content_obsolete"
    commands
        .entity(page_content_entity)
        .with_children(|parent| {
            draw_manual_page(parent, &handles, &manuals, &current_manual_page);
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
            (user_manual_system, redraw_manual_ui_system)
                .chain()
                .run_if(in_state(root::State::UserManual)),
        )
        .insert_resource(CurrentManualPage::default());
}
