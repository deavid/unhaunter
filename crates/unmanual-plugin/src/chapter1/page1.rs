use crate::types::ManualPageData;
use crate::utils::{grid_img_text2, header, summary_text};
use bevy::prelude::*;
use unmanual_core::assets::ManualAssets;
use unui_core::assets::UiAssets;

pub(crate) fn draw(
    parent: &mut ChildSpawnerCommands,
    manual_assets: &ManualAssets,
    ui_assets: &UiAssets,
) {
    let title = "Paranormal Investigator Needed!";
    let subtitle = "
Reports of unsettling activity... restless spirits... your expertise is required to expel the ghosts haunting these locations.
What will you find? How to do a good job as a P.I.? Here are the main clues!
".trim();
    let grid = vec![
        (
            &manual_assets.manual_investigate,
            "*1. Explore the location* and use your equipment (Thermometer, etc) to detect paranormal activity.",
        ),
        (
            &manual_assets.manual_locate_ghost,
            "*2. Find the breach*, which allows the ghost to haunt this location.",
        ),
        (
            &manual_assets.manual_identify_ghost,
            "*3. Different ghosts leave different evidence.* Recording your findings in the truck journal page helps you identify the ghost type.",
        ),
        (
            &manual_assets.manual_craft_repellent,
            "*4. Craft a repellent in the truck* for that particular ghost type once you know which ghost is.",
        ),
        (
            &manual_assets.manual_expel_ghost,
            "*5. Use the repellent to banish it* from the location. Confront the ghost.",
        ),
        (
            &manual_assets.manual_end_mission,
            "*6. Click 'End Mission'* in the truck to complete the investigation and receive your score.",
        ),
    ];
    let summary = "Your goal is to identify and banish the ghost. By exploring, locating the breach, gathering evidence, and using the truck's equipment, you'll craft the right tool for the job. More details on each step are provided in the following pages.";

    header(parent, ui_assets, title, subtitle);

    grid_img_text2(
        parent,
        &ui_assets.font_chakra_regular,
        &ui_assets.font_chakra_semibold,
        (3, 2),
        grid,
    );

    summary_text(parent, ui_assets, summary);
}

pub(crate) fn create_manual_page() -> ManualPageData {
    ManualPageData { draw_fn: draw }
}
