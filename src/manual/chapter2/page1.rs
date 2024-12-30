use crate::manual::utils::{grid_img_text2, header};
use crate::manual::ManualPageData;
use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Mastering UV and Night Vision Camera";
    let subtitle = "Understanding gear and seeing the invisible.";
    let grid = vec![
        (
            &handles.images.manual_uv_breach,
            "*1. Breach under UV:* The breach always glows golden under UV Light, making it easier to spot.",
        ),
        (
            &handles.images.manual_uv_ghost,
            "*2. UV Ectoplasm:* Some ghosts emit a greenish glow under UV light, which is evidence for *UV Ectoplasm*.",
        ),
        (
            &handles.images.manual_uv_object,
            "*3. Ghost Influence:* Certain objects emit a green glow under UV light. Ghosts are naturally attracted to these.",
        ),
        (
            &handles.images.manual_left_hand_videocam,
            "*4. Left Hand:* The Videocam can be used with *[TAB]* too if it's placed on the left hand slot."
        ),
        (
            &handles.images.manual_floating_orbs,
            "*5. Floating Orbs:* If you see that the breach glows bright white under Night Vision means that this ghost has *Floating Orbs* as evidence.",
        ),
        (
            &handles.images.manual_truck_refuge,
            "*6. Truck as Refuge:* Remember to return to the truck to prepare your investigation, rest and recover. *Sanity* and *Health* are recovered while on the truck.",
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
}

pub fn create_manual_page() -> ManualPageData {
    ManualPageData {
        title: "Mastering Inventory and UV".into(),
        subtitle: "Understanding gear and seeing the invisible.".into(),
        draw_fn: draw,
    }
}
