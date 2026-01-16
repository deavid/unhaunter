use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct ManualAssets {
    // Chapter 1
    #[asset(path = "manual/images/chapter1/investigate.png")]
    pub manual_investigate: Handle<Image>,
    #[asset(path = "manual/images/chapter1/locate_ghost.png")]
    pub manual_locate_ghost: Handle<Image>,
    #[asset(path = "manual/images/chapter1/identify_ghost.png")]
    pub manual_identify_ghost: Handle<Image>,
    #[asset(path = "manual/images/chapter1/craft_repellent.png")]
    pub manual_craft_repellent: Handle<Image>,
    #[asset(path = "manual/images/chapter1/expel_ghost.png")]
    pub manual_expel_ghost: Handle<Image>,
    #[asset(path = "manual/images/chapter1/end_mission.png")]
    pub manual_end_mission: Handle<Image>,
    #[asset(path = "manual/images/chapter1/movement_wasd.png")]
    pub manual_movement_wasd: Handle<Image>,
    #[asset(path = "manual/images/chapter1/interacting_objects.png")]
    pub manual_interacting_objects: Handle<Image>,
    #[asset(path = "manual/images/chapter1/flashlight.png")]
    pub manual_flashlight: Handle<Image>,
    #[asset(path = "manual/images/chapter1/activate_equipment.png")]
    pub manual_activate_equipment: Handle<Image>,
    #[asset(path = "manual/images/chapter1/switch_item.png")]
    pub manual_switch_item: Handle<Image>,
    #[asset(path = "manual/images/chapter1/quick_evidence.png")]
    pub manual_quick_evidence: Handle<Image>,
    #[asset(path = "manual/images/chapter1/emf_reader.png")]
    pub manual_emf_reader: Handle<Image>,
    #[asset(path = "manual/images/chapter1/thermometer.png")]
    pub manual_thermometer: Handle<Image>,
    #[asset(path = "manual/images/chapter1/truck_sanity.png")]
    pub manual_truck_sanity: Handle<Image>,
    #[asset(path = "manual/images/chapter1/ghost_attack.png")]
    pub manual_ghost_attack: Handle<Image>,
    #[asset(path = "manual/images/chapter1/identify_ghost.png")] // Used for truck journal
    pub manual_truck_journal: Handle<Image>,
    #[asset(path = "manual/images/chapter1/truck_exterior.png")]
    pub manual_truck_exterior: Handle<Image>,

    // Chapter 2
    #[asset(path = "manual/images/chapter2/left_hand_videocam.png")]
    pub manual_left_hand_videocam: Handle<Image>,
    #[asset(path = "manual/images/chapter2/uv_ghost.png")]
    pub manual_uv_ghost: Handle<Image>,
    #[asset(path = "manual/images/chapter2/uv_object.png")]
    pub manual_uv_object: Handle<Image>,
    #[asset(path = "manual/images/chapter2/uv_breach.png")]
    pub manual_uv_breach: Handle<Image>,
    #[asset(path = "manual/images/chapter2/floating_orbs.png")]
    pub manual_floating_orbs: Handle<Image>,
    #[asset(path = "manual/images/chapter2/inventory_all.png")]
    pub manual_inventory_all: Handle<Image>,
    #[asset(path = "manual/images/chapter2/ghost_red.png")]
    pub manual_ghost_red: Handle<Image>,
    #[asset(path = "manual/images/chapter2/ghost_roar.png")]
    pub manual_ghost_roar: Handle<Image>,
    #[asset(path = "manual/images/chapter2/hide_table.png")]
    pub manual_hide_table: Handle<Image>,
    #[asset(path = "manual/images/chapter2/truck_loadout.png")]
    pub manual_truck_loadout: Handle<Image>,
    #[asset(path = "manual/images/chapter2/truck_endmission.png")]
    pub manual_truck_endmission: Handle<Image>,
    #[asset(path = "manual/images/chapter2/truck_refuge.png")]
    pub manual_truck_refuge: Handle<Image>,

    // Chapter 3
    #[asset(path = "manual/images/chapter3/recorder_evp.png")]
    pub manual_recorder_evp: Handle<Image>,
    #[asset(path = "manual/images/chapter3/geiger_counter.png")]
    pub manual_geiger_counter: Handle<Image>,
    #[asset(path = "manual/images/chapter2/ghost_red.png")] // Locating ghost uses ch2 ghost_red
    pub manual_locating_ghost: Handle<Image>,
    #[asset(path = "manual/images/chapter3/sanity_management.png")]
    pub manual_sanity_management: Handle<Image>,
    #[asset(path = "manual/images/chapter1/emf_reader.png")]
    // EMF fluctuations uses ch1 emf_reader
    pub manual_emf_fluctuations: Handle<Image>,

    // Chapter 4
    #[asset(path = "manual/images/chapter4/object_interaction.png")]
    pub manual_object_interaction: Handle<Image>,
    #[asset(path = "manual/images/chapter4/object_interaction_2.png")]
    pub manual_object_interaction_2: Handle<Image>,
    #[asset(path = "manual/images/chapter4/spirit_box.png")]
    pub manual_spirit_box: Handle<Image>,
    #[asset(path = "manual/images/chapter4/red_torch.png")]
    pub manual_red_torch: Handle<Image>,

    // Chapter 5
    #[asset(path = "manual/images/chapter5/salt.png")]
    pub manual_salt: Handle<Image>,
    #[asset(path = "manual/images/chapter5/quartz.png")]
    pub manual_quartz: Handle<Image>,
    #[asset(path = "manual/images/chapter5/sage.png")]
    pub manual_sage: Handle<Image>,
}
