use super::super::ManualPageData;
use super::super::utils::{grid_img_text2, header, summary_text};
use bevy::prelude::*;
use uncore::types::root::game_assets::GameAssets;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Essential Controls";
    let subtitle = "
 Mastering the basics: movement, interaction, and illumination.
"
    .trim();
    let grid = vec![
        (
            &handles.images.manual_movement_wasd,
            "*1. Movement:* Use the *[W][A][S][D]* keys to move your character around the environment. Explore every corner of the haunted location!",
        ),
        (
            &handles.images.manual_interacting_objects,
            "*2. Interaction:* Press *[E]* to interact with objects like doors, light switches, and furniture. Uncover clues and manipulate the environment to your advantage.",
        ),
        (
            &handles.images.manual_flashlight,
            "*3. Flashlight:* Press *[Tab]* to toggle your flashlight on and off. Illuminate the darkness and reveal what lurks in the shadows. But be mindful of overheating!",
        ),
        (
            &handles.images.manual_activate_equipment,
            "*4. Right-Hand Gear:* Press *[R]* to activate the equipment in your right hand. Gather evidence and unravel the mysteries of the haunting.",
        ),
        (
            &handles.images.manual_switch_item,
            "*5. Inventory Cycling (Right Hand):* Press *[Q]* to cycle through the items stored in your right hand's inventory slots. Quickly switch between essential tools.",
        ),
        // TODO: [F] and [G] keys are to be explained in chapter2. In there we should also explain [T] Swap hands, and the hiding mechanic.
        (
            &handles.images.manual_quick_evidence,
            "*6.  Quick Evidence:* Press *[C]* to mark the current evidence type displayed by the equipment in your right hand. Quickly tag the evidence you've found without needing to return to the truck.",
        ),
    ];
    let summary = "These controls are essential for navigating the haunted locations, gathering evidence, and ultimately expelling the ghost. Experiment with your equipment and learn how to use your environment for a successful investigation.";

    header(parent, handles, title, subtitle);

    grid_img_text2(
        parent,
        &handles.fonts.chakra.w400_regular,
        &handles.fonts.chakra.w600_semibold,
        (3, 2),
        grid,
    );

    summary_text(parent, handles, summary);
}

pub fn create_manual_page() -> ManualPageData {
    ManualPageData {
        title: "Essential Controls".into(),
        subtitle: "Mastering the basics: movement, interaction, and illumination.".into(),
        draw_fn: draw,
    }
}
