use super::super::ManualPageData;
use super::super::utils::{grid_img_text2, header, summary_text};
use bevy::prelude::*;
use uncore::types::root::game_assets::GameAssets;

pub fn draw(parent: &mut ChildSpawnerCommands, handles: &GameAssets) {
    let title = "Tools of the Adept";
    let subtitle = "Learn to use Salt, Quartz, and Sage to handle paranormal threats.";
    let grid = vec![
        (
            &handles.images.manual_salt,
            "*1. Salt:* Drop salt piles so ghosts leave UV-visible trails when a ghost moves across the salt, enabling you to track the ghost.",
        ),
        (&handles.images.manual_salt, "N/A"),
        (
            &handles.images.manual_quartz,
            "*2. Quartz:* The Quartz Stone absorbs energy from the ghost when hunting close to it, shortening the hunt. The stone will gradually crack, shattering and becoming useless.",
        ),
        (&handles.images.manual_salt, "N/A"),
        (
            &handles.images.manual_sage,
            "*3. Sage:* Sage is a consumable item that emits smoke when activated. This smoke has a calming effect on ghosts, delaying their hunt or making them lose track of the player during an active hunt.",
        ),
        (&handles.images.manual_salt, "N/A"),
    ];

    let summary = "By mastering these new items you are now ready for a new level of investigation. Good luck.";

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
        title: "Tools of the Adept".into(),
        subtitle: "Learn to use Salt, Quartz, and Sage to handle paranormal threats.".into(),
        draw_fn: draw,
    }
}
