use crate::manual::utils::{grid_img_text2, header, summary_text};

use crate::manual::ManualPageData;
use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "The Truck: Your Ghost Hunting HQ";
    let subtitle = "
Gather evidence, analyze your findings, and prepare for the unknown. 
"
    .trim();
    let grid = vec![
        (
            &handles.images.manual_emf_reader,
            "*1. EMF Reader:* The EMF reader detects electromagnetic changes, which can indicate a ghost's presence. Hold it near suspected areas of ghost activity. A reading of *EMF5* on device is strong evidence."
        ),
        (
            &handles.images.manual_thermometer,
            "*2. Thermometer:* The thermometer measures temperature. Some ghosts cause temperatures to drop significantly, even below freezing. Use the thermometer to find these cold spots, if it reads *below zero*, mark it as evidence."
        ),
        (
            &handles.images.manual_truck_sanity,
            "*3. Sanity:* Staying in the dark or being exposed to the ghost's presence for too long will gradually decrease your Sanity. Low Sanity can have negative effects! While in the truck your sanity will recover gradually."
        ),
        (
            &handles.images.manual_ghost_attack,
            "*4. Ghost Attack:* Be careful! The ghost might become aggressive and attack. If the ghost turns red, run away! Low Sanity will lead to the ghost attacking more often.",
        ),
        (
            &handles.images.manual_truck_exterior,
            "*5. The Truck: Your Safe Haven:* You're safe inside your truck, parked outside the haunted location. Your Sanity level will recover over time here."
        ),
        (
            &handles.images.manual_truck_journal,
            "*6. Truck Journal:* In the \"Journal\" tab, click the evidence buttons to mark which ones you've found. The journal will then filter the possible ghost types based on your selections.",
        ),
    ];
    let summary = "Mastering these tools, understanding the truck's importance, and managing your sanity are crucial for a successful ghost hunt.";

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
        title: "The Truck: Your Ghost Hunting HQ".into(),
        subtitle: "Gather evidence, analyze your findings, and prepare for the unknown.".into(),
        draw_fn: draw,
    }
}
