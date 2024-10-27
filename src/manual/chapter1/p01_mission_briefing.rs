use bevy::prelude::*;

use super::super::utils::image_text;
use crate::platform::plt::UI_SCALE;
use crate::root::GameAssets;

pub fn draw_mission_briefing_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    // Mission Briefing Section
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0), // Occupy full width
                height: Val::Percent(10.0),
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
                    font_size: 48.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));

            // Premise Text
            parent.spawn(TextBundle::from_section(
                "Reports of unsettling activity... restless spirits... your expertise is required to investigate and expel the ghosts haunting these locations.",
                TextStyle {
                    font: handles.fonts.chakra.w400_regular.clone(),
                    font_size: 24.0 * UI_SCALE,
                    color: Color::WHITE,
                },
            ));
        });

    // Gameplay Loop Section (3x2 Grid)
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0), // Occupy full width
                height: Val::Percent(70.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(3, 1.0), // 3 equal-width columns
                grid_template_rows: RepeatedGridTrack::percent(2, 60.0),
                column_gap: Val::Percent(4.0),
                row_gap: Val::Percent(4.0),
                padding: UiRect::all(Val::Percent(2.0)),
                ..default()
            },

            ..default()
        })
        .with_children(|parent| {
            // Step 1: Investigate
            image_text(
                parent,
                &handles.images.manual_investigate,
                &handles.fonts.chakra.w400_regular,
                "Explore the location, search for clues, and use your equipment to detect paranormal activity.",
            );

            // Step 2: Locate Ghost
            image_text(
                parent,
                &handles.images.manual_locate_ghost,
                &handles.fonts.chakra.w400_regular,
                "Find the ghost's breach, a subtle dust cloud that marks its presence.",
            );

            // Step 3: Identify Ghost
            image_text(
                parent,
                &handles.images.manual_identify_ghost,
                &handles.fonts.chakra.w400_regular,
                "Gather evidence using your equipment and record your findings in the truck journal to identify the ghost type.",
            );

            // Step 4: Craft Repellent
            image_text(
                parent,
                &handles.images.manual_craft_repellent,
                &handles.fonts.chakra.w400_regular,
                "Once you've identified the ghost, craft a specialized repellent in the truck.",
            );

            // Step 5: Expel Ghost
            image_text(
                parent,
                &handles.images.manual_expel_ghost,
                &handles.fonts.chakra.w400_regular,
                "Confront the ghost and use the repellent to banish it from the location.",
            );

            // Step 6: End Mission
            image_text(
                parent,
                &handles.images.manual_end_mission,
                &handles.fonts.chakra.w400_regular,
                "Return to the truck and click 'End Mission' to complete the investigation and receive your score.",
            );

        });
}
