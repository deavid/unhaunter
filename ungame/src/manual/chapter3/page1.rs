use crate::manual::utils::{grid_img_text2, header, summary_text};
use crate::manual::ManualPageData;
use crate::root::GameAssets;
use bevy::prelude::*;

pub fn draw(parent: &mut ChildBuilder, handles: &GameAssets) {
    let title = "Mastering Advanced Investigation";
    let subtitle = "
        Delve deeper into the paranormal, utilizing advanced techniques and specialized gear to uncover the unseen.
        "
    .trim();
    let grid = vec![
        (
            &handles.images.manual_recorder_evp,
            "*1. New Evidence: EVP Recordings (Recorder):* Use the Recorder to capture Electronic Voice Phenomena (EVP) - ghostly voices. Press *[R]* to use. Get close to the ghost to get a better reading. *EVP RECORDED* indicates a successful capture. Combine this with other evidence to identify the ghost."
        ),
        (
            &handles.images.manual_geiger_counter,
            "*2. New Evidence: 500+ cpm (Geiger Counter):* The Geiger Counter measures radiation. Some ghosts emit high radiation, registering *over 500 cpm* (counts per minute). The closer to the radiation, the faster the 'click' and the higher the reading. Use it to locate ghosts on big maps."
        ),
        (
            &handles.images.manual_locating_ghost,
            "*3. Locating the Ghost:* Combine your Thermometer, EMF, and Geiger to pinpoint the ghost. The ghost will make the ambient colder and will also emit sounds, use the readings to find it. Remember that the values fluctuate, take your time."
        ),
        (
            &handles.images.manual_quick_evidence,
            "*4. Mark Evidence Remotely:* Press *[C]* to mark the evidence type currently displayed by your equipment. This will record it in your journal's evidence tracker without needing to return to the truck immediately."
        ),
        (
            &handles.images.manual_sanity_management,
            "*5. Managing Your Sanity:* Keep lights on in the rooms you are investigating *(Darkness drains your sanity)*, and use the truck to recover. *Note:* Your flashlight does *not* prevent sanity loss, only room lights do."
        ),
        (
            &handles.images.manual_emf_fluctuations,
            "*6. Ghost Behavior: EMF Fluctuations:* Ghosts can cause fluctuations in the EMF even when not giving an EMF5 reading. Watch for sudden spikes or changes in the EMF reading, it could indicate the ghost's presence or movement, even if it doesn't reach level 5.",
        ),
    ];
    let summary = "Master these techniques, manage your sanity, and learn to interpret the subtle signs. You're ready to face the challenges ahead. Good luck, Senior Investigator!";

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
        title: "Mastering Advanced Investigation".into(),
        subtitle: "Delve deeper into the paranormal, utilizing advanced techniques and specialized gear to uncover the unseen.".into(),
        draw_fn: draw,
    }
}
