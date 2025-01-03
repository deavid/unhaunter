use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct ImageAssets {
    pub title: Handle<Image>,
    pub character1: Handle<Image>,
    pub gear: Handle<Image>,
    pub character1_atlas: Handle<TextureAtlasLayout>,
    pub gear_atlas: Handle<TextureAtlasLayout>,
    pub vignette: Handle<Image>,
    // --- Manual Images ---
    // Chapter 1: Page 1:
    pub manual_investigate: Handle<Image>,
    pub manual_locate_ghost: Handle<Image>,
    pub manual_identify_ghost: Handle<Image>,
    pub manual_craft_repellent: Handle<Image>,
    pub manual_expel_ghost: Handle<Image>,
    pub manual_end_mission: Handle<Image>,
    // Chapter 1: Page 2:
    pub manual_movement_wasd: Handle<Image>,
    pub manual_interacting_objects: Handle<Image>,
    pub manual_flashlight: Handle<Image>,
    pub manual_activate_equipment: Handle<Image>,
    pub manual_switch_item: Handle<Image>,
    pub manual_quick_evidence: Handle<Image>,
    // Chapter 1: Page 3:
    pub manual_emf_reader: Handle<Image>,
    pub manual_thermometer: Handle<Image>,
    pub manual_truck_sanity: Handle<Image>,
    pub manual_ghost_attack: Handle<Image>,
    pub manual_truck_journal: Handle<Image>,
    pub manual_truck_exterior: Handle<Image>,
    // --- Chapter 2 images ---
    pub manual_left_hand_videocam: Handle<Image>,
    pub manual_uv_ghost: Handle<Image>,
    pub manual_uv_object: Handle<Image>,
    pub manual_uv_breach: Handle<Image>,
    pub manual_floating_orbs: Handle<Image>,
    pub manual_inventory_all: Handle<Image>,
    pub manual_ghost_red: Handle<Image>,
    pub manual_ghost_roar: Handle<Image>,
    pub manual_hide_table: Handle<Image>,
    pub manual_truck_loadout: Handle<Image>,
    pub manual_truck_endmission: Handle<Image>,
    pub manual_truck_refuge: Handle<Image>,
    // -- Chapter 3 Images
    pub manual_recorder_evp: Handle<Image>,
    pub manual_geiger_counter: Handle<Image>,
    pub manual_locating_ghost: Handle<Image>,
    pub manual_sanity_management: Handle<Image>,
    pub manual_emf_fluctuations: Handle<Image>,
    // -- Chapter 4 Images
    pub manual_object_interaction: Handle<Image>,
    pub manual_object_interaction_2: Handle<Image>,
    pub manual_spirit_box: Handle<Image>,
    pub manual_red_torch: Handle<Image>,
    // -- Chapter 5 Images
    pub manual_salt: Handle<Image>,
    pub manual_quartz: Handle<Image>,
    pub manual_sage: Handle<Image>,
}
