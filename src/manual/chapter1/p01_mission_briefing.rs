use crate::manual::utils::{grid_img_text, header};

use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw_mission_briefing_page(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Paranormal Investigator Needed!";
    let subtitle = "Reports of unsettling activity... restless spirits... your expertise is required to investigate and expel the ghosts haunting these locations.";
    let grid = vec![
        (
            &handles.images.manual_investigate,
            "Explore the location, search for clues, and use your equipment to detect paranormal activity."
        ),
        (
            &handles.images.manual_locate_ghost,
          "Find the ghost's breach, a subtle dust cloud that marks its presence.",
        ),
        (
            &handles.images.manual_identify_ghost,
            "Gather evidence using your equipment and record your findings in the truck journal to identify the ghost type.",
        ),
        (
            &handles.images.manual_craft_repellent,
            "Once you've identified the ghost, craft a specialized repellent in the truck.",
        ),
        (
            &handles.images.manual_expel_ghost,
            "Confront the ghost and use the repellent to banish it from the location.",
        ),
        (
            &handles.images.manual_end_mission,
            "Return to the truck and click 'End Mission' to complete the investigation and receive your score.",
        ),
    ];

    header(parent, handles, title, subtitle);

    grid_img_text(parent, &handles.fonts.chakra.w400_regular, (3, 2), grid);
}
