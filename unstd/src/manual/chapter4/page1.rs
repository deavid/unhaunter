use super::super::ManualPageData;
use super::super::utils::{grid_img_text2, header, summary_text};
use bevy::prelude::*;
use uncore::types::root::game_assets::GameAssets;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Expanding Your Arsenal";
    let subtitle = "
        Delve deeper into the paranormal, utilizing advanced techniques and specialized gear to uncover the unseen.
    ".trim();
    let grid = vec![
        (
            &handles.images.manual_spirit_box,
            "*1. New Gear: Spirit Box:* A modified AM Radio that constantly scans through radio frequencies. If the ghosts talk though it, this is the evidence **Spirit Box**.  It is best if used near the breach and in darkness.",
        ),
        (
            &handles.images.manual_red_torch,
            "*2. New Gear: Red Torch:* Emits a special red light that certain ghost types will glow golden, this is the evidence **RL Presence**.",
        ),
        (
            &handles.images.manual_uv_object,
            "*3. Interacting with Objects (I):* Objects in the environment can influence the ghost. Use the **UV Torch**, **Red Torch** and **Video Cam** to identify them. Attractive objects glow green under UV, while repulsive objects glow blue under the Red Torch. Both glow the same while using the Video Cam.",
        ),
        (
            &handles.images.manual_object_interaction,
            "*4. Interacting with Objects (II):* You can **move** some objects by pressing the **[F]** key. Move Attractive objects closer to the ghost's suspected location to lure it or place Repulsive objects to create barriers.",
        ),
        (
            &handles.images.manual_object_interaction_2,
            "*5. Object Charge and Ghost Rage:* Removing an **Attractive** object from the location or placing a **Repulsive** object near the ghost's breach will significantly increase its rage, potentially triggering a hunt.",
        ),
        (
            &handles.images.manual_quick_evidence,
            "*6. Evidence: Spirit Box and RL Presence:* The **Spirit Box** and **Red Torch** are linked to specific evidence types. If a ghost responds through the Spirit Box or reacts to the Red Light, mark it as evidence using **[C]**.",
        ),
    ];

    header(parent, handles, title, subtitle);

    grid_img_text2(
        parent,
        &handles.fonts.chakra.w400_regular,
        &handles.fonts.chakra.w600_semibold,
        (3, 2),
        grid,
    );

    summary_text(
        parent,
        handles,
        "Experiment with the new gear, observe the ghost's reactions, and use your environment to your advantage.",
    );
}

pub fn create_manual_page() -> ManualPageData {
    ManualPageData {
        title: "Expanding Your Arsenal".into(),
        subtitle: "Learn about the Spirit Box, Red Torch, and object interactions.".into(),
        draw_fn: draw,
    }
}
