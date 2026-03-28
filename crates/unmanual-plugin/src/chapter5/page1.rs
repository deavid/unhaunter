use crate::types::ManualPageData;
use crate::utils::{grid_img_text2, header, summary_text};
use bevy::prelude::*;
use unmanual_core::assets::ManualAssets;

pub(crate) fn draw(
    parent: &mut ChildSpawnerCommands,
    manual_assets: &ManualAssets,
    _second_manual_assets: &ManualAssets,
) {
    let title = "Tools of the Adept";
    let subtitle = "Learn to use Salt, Quartz, and Sage to handle paranormal threats.";
    let grid = vec![
        (
            &manual_assets.manual_salt,
            "*1. Salt:* Drop salt piles so ghosts leave UV-visible trails when a ghost moves across the salt, enabling you to track the ghost.",
        ),
        (&manual_assets.manual_salt, "N/A"),
        (
            &manual_assets.manual_quartz,
            "*2. Quartz:* The Quartz Stone absorbs energy from the ghost when hunting close to it, shortening the hunt. The stone will gradually crack, shattering and becoming useless.",
        ),
        (&manual_assets.manual_salt, "N/A"),
        (
            &manual_assets.manual_sage,
            "*3. Sage:* Sage is a consumable item that emits smoke when activated. This smoke has a calming effect on ghosts, delaying their hunt or making them lose track of the player during an active hunt.",
        ),
        (&manual_assets.manual_salt, "N/A"),
    ];

    let summary = "By mastering these new items you are now ready for a new level of investigation. Good luck.";

    header(parent, manual_assets, title, subtitle);

    grid_img_text2(
        parent,
        &manual_assets.font_chakra_regular,
        &manual_assets.font_chakra_semibold,
        (3, 2),
        grid,
    );

    summary_text(parent, manual_assets, summary);
}

pub(crate) fn create_manual_page() -> ManualPageData {
    ManualPageData { draw_fn: draw }
}
