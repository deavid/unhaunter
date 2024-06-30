// src/manual/chapter1/mission_briefing.rs

use bevy::prelude::*;

use crate::root::GameAssets;

pub fn draw_mission_briefing_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    // Mission Briefing Section
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Headline
            parent.spawn(TextBundle::from_section(
                "Paranormal Investigator Needed!",
                TextStyle {
                    font: handles.fonts.londrina.w300_light.clone(),
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            ));

            // Premise Text
            parent.spawn(TextBundle::from_section(
                "Reports of unsettling activity... restless spirits... your expertise is required to investigate and expel the ghosts haunting these locations.",
                TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ));
        });

    // Gameplay Loop Section
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(70.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Column 1
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(33.3),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Investigate
                    parent.spawn(ImageBundle {
                        style: Style {
                            max_width: Val::Px(256.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            aspect_ratio: Some(4.0/3.0),
                            ..default()
                        },
                        image: handles.images.manual_investigate.clone().into(), // Image path
                        ..default()
                    });
                    parent.spawn(TextBundle::from_section(
                        "Explore the location, search for clues, and use your equipment to detect paranormal activity.",
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ));

                    // Locate Ghost
                    parent.spawn(ImageBundle {
                        style: Style {
                            max_width: Val::Px(256.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            aspect_ratio: Some(4.0/3.0),
                            ..default()
                        },
                        image: handles.images.manual_locate_ghost.clone().into(), // Image path
                        ..default()
                    });
                    parent.spawn(TextBundle::from_section(
                        "Find the ghost's breach, a subtle dust cloud that marks its presence.",
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            // Column 2
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(33.3),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Identify Ghost
                    parent.spawn(ImageBundle {
                        style: Style {
                            max_width: Val::Px(256.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            aspect_ratio: Some(4.0/3.0),
                            ..default()
                        },
                        image: handles.images.manual_identify_ghost.clone().into(), // Image path
                        ..default()
                    });
                    parent.spawn(TextBundle::from_section(
                        "Gather evidence using your equipment and record your findings in the truck journal to identify the ghost type.",
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ));

                    // Craft Repellent
                    parent.spawn(ImageBundle {
                        style: Style {
                            max_width: Val::Px(256.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            aspect_ratio: Some(4.0/3.0),
                            ..default()
                        },
                        image: handles.images.manual_craft_repellent.clone().into(), // Image path
                        ..default()
                    });
                    parent.spawn(TextBundle::from_section(
                        "Once you've identified the ghost, craft a specialized repellent in the truck.",
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ));
                });

            // Column 3
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(33.3),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Expel Ghost
                    parent.spawn(ImageBundle {
                        style: Style {
                            max_width: Val::Px(256.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            aspect_ratio: Some(4.0/3.0),
                            ..default()
                        },
                        image: handles.images.manual_expel_ghost.clone().into(), // Image path
                        ..default()
                    });
                    parent.spawn(TextBundle::from_section(
                        "Confront the ghost and use the repellent to banish it from the location.",
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ));

                    // End Mission
                    parent.spawn(ImageBundle {
                        style: Style {
                            max_width: Val::Px(256.0),
                            margin: UiRect::all(Val::Px(10.0)),   
                            aspect_ratio: Some(4.0/3.0),
                            ..default()
                        },
                        image: handles.images.manual_end_mission.clone().into(), // Image path

                        ..default()
                    });
                    parent.spawn(TextBundle::from_section(
                        "Return to the truck and click 'End Mission' to complete the investigation and receive your score.",
                        TextStyle {
                            font: handles.fonts.chakra.w400_regular.clone(),
                            font_size: 18.0,
                            color: Color::WHITE,
                        },
                    ));
                });
        });
}
