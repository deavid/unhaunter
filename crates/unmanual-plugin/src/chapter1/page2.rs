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
    let title = "Essential Controls";
    let subtitle = "
 Mastering the basics: movement, interaction, and illumination.
"
    .trim();
    let grid = vec![
        (
            &manual_assets.manual_movement_wasd,
            "*1. Movement:* Use the *[W][A][S][D]* keys to move your character around the environment. Explore every corner of the haunted location!",
        ),
        (
            &manual_assets.manual_interacting_objects,
            "*2. Interaction:* Press *[E]* to interact with objects like doors, light switches, and furniture. Uncover clues and manipulate the environment to your advantage.",
        ),
        (
            &manual_assets.manual_flashlight,
            "*3. Flashlight:* Press *[Tab]* to toggle your flashlight on and off. Illuminate the darkness and reveal what lurks in the shadows. But be mindful of overheating!",
        ),
        (
            &manual_assets.manual_activate_equipment,
            "*4. Right-Hand Gear:* Press *[R]* to activate the equipment in your right hand. Gather evidence and unravel the mysteries of the haunting.",
        ),
        (
            &manual_assets.manual_switch_item,
            "*5. Inventory Cycling (Right Hand):* Press *[Q]* to cycle through the items stored in your right hand's inventory slots. Quickly switch between essential tools.",
        ),
        // TODO: [F] and [G] keys are to be explained in chapter2. In there we should also explain [T] Swap hands, and the hiding mechanic.
        (
            &manual_assets.manual_quick_evidence,
            "*6.  Quick Evidence:* Press *[C]* to mark the current evidence type displayed by the equipment in your right hand. Quickly tag the evidence you've found without needing to return to the truck.",
        ),
    ];
    let summary = "These controls are essential for navigating the haunted locations, gathering evidence, and ultimately expelling the ghost. Experiment with your equipment and learn how to use your environment for a successful investigation.";

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
