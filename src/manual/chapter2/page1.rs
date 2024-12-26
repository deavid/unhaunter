use crate::manual::utils::{grid_img_text2, header};
use crate::manual::ManualPageData;
use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Mastering Inventory and UV";
    let subtitle = "Understanding gear and seeing the invisible.";
    let grid = vec![
        (
            &handles.images.manual_quick_change,
            "*1. Quick Switch:* Use *[Q]* to quickly cycle through the gear available in your right hand."
        ),
        (
            &handles.images.manual_uv_ghost,
            "*2. UV Ectoplasm:* Some ghosts emit a greenish glow under UV light, which is a strong piece of evidence.",
        ),
        (
            &handles.images.manual_uv_object,
            "*3. Ghost Influence:* Certain objects emit a green glow under UV light. Ghosts are naturally attracted to these.",
        ),
        (
            &handles.images.manual_torch_tab,
            "*4. Left Hand:* Use *[TAB]* to quickly toggle your left-hand equipment (mostly the flashlight) on or off.",
        ),
        (
            &handles.images.manual_inventory_slots,
            "*5. Limited Storage:* You have two slots for the items you equip on your hands, plus two more slots to keep some useful gear ready in your backpack.",
        ),
        (
            &handles.images.manual_inventory_all,
            "*6. Be organized!*: Plan which items to keep available, as some can be placed around to assist in your investigation.",
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
