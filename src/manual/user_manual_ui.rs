use super::{draw_manual_page, CurrentManualPage, Manual};
use crate::root::{self, GameAssets};
use bevy::prelude::*;

#[derive(Component)]
pub struct ManualCamera;

#[derive(Component)]
pub struct UserManualUI;

#[derive(Component)]
pub struct PageContent;

#[derive(Debug, Clone, Copy, Event)]
pub enum ManualNavigationEvent {
    NextPage,
    PreviousPage,
    Close,
}

pub fn draw_manual_ui(commands: &mut Commands, handles: Res<GameAssets>) {
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
                .insert(PageContent);

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
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ev_navigation: EventWriter<ManualNavigationEvent>,
    mut interaction_query: Query<
        (Ref<Interaction>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    text_query: Query<&Text>,
) {
    for (interaction, children) in &mut interaction_query {
        if interaction.is_changed() && *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok(text) = text_query.get(*child) {
                    match text.sections[0].value.as_str() {
                        "Previous" => {
                            ev_navigation.send(ManualNavigationEvent::PreviousPage);
                        }
                        "Next" => {
                            ev_navigation.send(ManualNavigationEvent::NextPage);
                        }
                        "Close" => {
                            ev_navigation.send(ManualNavigationEvent::Close);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        ev_navigation.send(ManualNavigationEvent::PreviousPage);
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        ev_navigation.send(ManualNavigationEvent::NextPage);
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        ev_navigation.send(ManualNavigationEvent::Close);
    }
}

pub fn setup(mut commands: Commands, handles: Res<GameAssets>) {
    // Spawn the 2D camera for the manual UI
    commands
        .spawn(Camera2dBundle::default())
        .insert(ManualCamera);

    // Draw the manual UI
    draw_manual_ui(&mut commands, handles);
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

/// Handles manual navigation events.
fn handle_manual_navigation(
    mut ev_navigation: EventReader<ManualNavigationEvent>,
    mut current_manual_page: ResMut<CurrentManualPage>,
    manuals: Res<Manual>,
    mut next_state: ResMut<NextState<root::State>>,
) {
    for ev in ev_navigation.read() {
        match ev {
            ManualNavigationEvent::PreviousPage => {
                if current_manual_page.1 > 0 {
                    current_manual_page.1 -= 1;
                } else if current_manual_page.0 > 0 {
                    current_manual_page.0 -= 1; // Go to the previous chapter
                    current_manual_page.1 = manuals.chapters[current_manual_page.0].pages.len() - 1;
                } else {
                    warn!("Already at the beginning of the manual");
                }
            }

            ManualNavigationEvent::NextPage => {
                let current_chapter_size = manuals.chapters[current_manual_page.0].pages.len();
                if current_manual_page.1 + 1 < current_chapter_size {
                    current_manual_page.1 += 1;
                } else if current_manual_page.0 + 1 < manuals.chapters.len() {
                    current_manual_page.0 += 1; // Go to the next chapter
                    current_manual_page.1 = 0;
                } else {
                    warn!("Already at the end of the manual");
                }
            }

            ManualNavigationEvent::Close => next_state.set(root::State::MainMenu),
        }
    }
}

/// System to control the visibility of navigation buttons in the manual.
fn update_navigation_button_visibility(
    current_manual_page: Res<CurrentManualPage>,
    manuals: Res<Manual>,
    mut button_query: Query<(&Children, &mut Visibility), With<Button>>,
    text_query: Query<&Text>,
) {
    for (children, mut visibility) in &mut button_query {
        for child in children.iter() {
            if let Ok(text) = text_query.get(*child) {
                let is_first = text.sections[0].value == "Previous"
                    && current_manual_page.1 == 0
                    && current_manual_page.0 == 0;
                let current_chapter_size = manuals.chapters[current_manual_page.0].pages.len();
                let is_last = text.sections[0].value == "Next"
                    && current_manual_page.1 + 1 == current_chapter_size
                    && current_manual_page.0 + 1 == manuals.chapters.len();

                *visibility = if is_first || is_last {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
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
            (
                user_manual_system,
                handle_manual_navigation,
                update_navigation_button_visibility,
                redraw_manual_ui_system,
            )
                .chain()
                .run_if(in_state(root::State::UserManual)),
        ) // Add event handler system
        .add_event::<ManualNavigationEvent>() // Register the event
        .insert_resource(CurrentManualPage::default());
}
