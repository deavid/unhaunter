use super::super::utils::{grid_img_text2, header};
use super::super::ManualPageData;
use bevy::prelude::*;
use uncore::types::root::game_assets::GameAssets;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Ghost Hunts and the Truck";
    let subtitle = "Surviving the paranormal and using your truck as your headquarters.";
    let grid = vec![
        (
            &handles.images.manual_ghost_red,
            "*1. Ghost's Hunt:* The ghost may become aggressive and start a hunt. This is a very dangerous state.",
        ),
        (
            &handles.images.manual_ghost_roar,
            "*2. Loud Ghost Roar:* Before a hunt starts, the ghost will make a loud, angry roar, giving you a hint of what's coming.",
        ),
        (
            &handles.images.manual_hide_table,
            "*3. Hiding Places:* If a hunt starts, hold *[E]* for a second to hide behind tables or beds, hoping the ghost will not find you.",
        ),
        (
            &handles.images.manual_truck_loadout,
            "*4. Select Your Equipment:* You can select which equipment you want to take with you from the *Loadout* tab of the truck before starting the investigation.",
        ),
        (
            &handles.images.manual_inventory_all,
            "*5. Be organized!*: The spots in your inventory are for the Left Hand *[TAB]*, Right Hand *[R]*, and two extra backpack slots *[Q]*.",
        ),
        (
            &handles.images.manual_truck_endmission,
             "*6. End Mission:* When you are sure that you have expelled all of the ghosts, click \"End Mission\" on the truck to receive your score.",
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
        title: "Ghost Hunts and the Truck".into(),
        subtitle: "Surviving the paranormal and using your truck as your headquarters.".into(),
        draw_fn: draw,
    }
}
