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
    let title = "Expanding Your Arsenal";
    let subtitle = "
        Delve deeper into the paranormal, utilizing advanced techniques and specialized gear to uncover the unseen.
    ".trim();
    let grid = vec![
        (
            &manual_assets.manual_spirit_box,
            "*1. New Gear: Spirit Box:* A modified AM Radio that constantly scans through radio frequencies. If the ghosts talk though it, this is the evidence **Spirit Box**.  It is best if used near the breach and in darkness.",
        ),
        (
            &manual_assets.manual_red_torch,
            "*2. New Gear: Red Torch:* Emits a special red light that certain ghost types will glow golden, this is the evidence **RL Presence**.",
        ),
        (
            &manual_assets.manual_uv_object,
            "*3. Interacting with Objects (I):* Objects in the environment can influence the ghost. Use the **UV Torch**, **Red Torch** and **Video Cam** to identify them. Attractive objects glow green under UV, while repulsive objects glow blue under the Red Torch. Both glow the same while using the Video Cam.",
        ),
        (
            &manual_assets.manual_object_interaction,
            "*4. Interacting with Objects (II):* You can **move** some objects by pressing the **[F]** key. Move Attractive objects closer to the ghost's suspected location to lure it or place Repulsive objects to create barriers.",
        ),
        (
            &manual_assets.manual_object_interaction_2,
            "*5. Object Charge and Ghost Rage:* Removing an **Attractive** object from the location or placing a **Repulsive** object near the ghost's breach will significantly increase its rage, potentially triggering a hunt.",
        ),
        (
            &manual_assets.manual_quick_evidence,
            "*6. Evidence: Spirit Box and RL Presence:* The **Spirit Box** and **Red Torch** are linked to specific evidence types. If a ghost responds through the Spirit Box or reacts to the Red Light, mark it as evidence using **[C]**.",
        ),
    ];

    header(parent, ui_assets, title, subtitle);

    grid_img_text2(
        parent,
        &ui_assets.font_chakra_regular,
        &ui_assets.font_chakra_semibold,
        (3, 2),
        grid,
    );

    summary_text(
        parent,
        ui_assets,
        "Experiment with the new gear, observe the ghost's reactions, and use your environment to your advantage.",
    );
}

pub(crate) fn create_manual_page() -> ManualPageData {
    ManualPageData { draw_fn: draw }
}
