use crate::manual::utils::{grid_img_text, header};

use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw_mission_briefing_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Paranormal Investigator Needed!";
    let subtitle = "
Reports of unsettling activity... restless spirits... your expertise is required to expel the ghosts haunting these locations.
What will you find? How to do a good job as a P.I.? Here are the main clues!
".trim();
    let grid = vec![
        (
            &handles.images.manual_investigate,
            "1. Explore the location and use your equipment (EMF, etc) to detect paranormal activity."
        ),
        (
            &handles.images.manual_locate_ghost,
            "2. Find the breach, which allows the ghost to haunt this location.",
        ),
        (
            &handles.images.manual_identify_ghost,
            "3. Record your findings in the truck journal page to identify the ghost type.",
        ),
        (
            &handles.images.manual_craft_repellent,
            "4. Once you know which ghost is, craft a repellent in the truck for that particular ghost type.",
        ),
        (
            &handles.images.manual_expel_ghost,
            "5. Confront the ghost and use the repellent to banish it from the location.",
        ),
        (
            &handles.images.manual_end_mission,
            "6. Return to the truck and click 'End Mission' to complete the investigation and receive your score.",
        ),
    ];

    header(parent, handles, title, subtitle);

    grid_img_text(parent, &handles.fonts.chakra.w400_regular, (3, 2), grid);
}
