use bevy::prelude::*;

use crate::root::GameAssets;

use super::{chapter1, ManualPage};

#[derive(Component)]
pub struct ManualUI;

pub fn draw_manual_page(parent: &mut ChildBuilder, handles: &GameAssets, current_page: ManualPage) {
    // Draw page-specific content
    match current_page {
        ManualPage::Introduction => chapter1::introduction::draw_introduction_page(parent, handles),
        ManualPage::BasicControls => {
            chapter1::basic_controls::draw_basic_controls_page(parent, handles)
        }
    }
}

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
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ..default()
        })
        .insert(ManualUI)
        .with_children(|parent| {
            // Page Content Container
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|content| {
                    draw_manual_page(content, &handles, *current_page);
                });

            // Navigation Buttons
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(10.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|buttons| {
                    // Previous Button
                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Percent(40.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::BLACK.with_a(0.2).into(),
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
                                width: Val::Percent(40.0),
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::BLACK.with_a(0.2).into(),
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

            // Close Button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(10.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::BLACK.with_a(0.2).into(),
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
        });
}
