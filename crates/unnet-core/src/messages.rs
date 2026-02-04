use crate::network_id::NetworkId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unevents_core::events::roomchanged::InteractionExecutionType;
use unfoundation_core::types::grade::Grade;
use ungearitems_core::components::flashlight::FlashlightStatus;
use unghost_core::components::ghost_influence::InfluenceType;
use unghost_core::types::evidence::Evidence;
use unghost_core::types::ghost::types::GhostType;
use untypes_core::states::{AppState, GameState};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GearDetails {
    Flashlight(FlashlightStatus),
    Thermometer {
        temp: f32,
    },
    EMF {
        level: f32,
    },
    Sage {
        consumed: bool,
        is_active: bool,
        remaining_secs: f32,
    },
    RepellentFlask {
        qty: i32,
        active: bool,
    },
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: NetworkId,
    pub position: [f32; 3],
    pub orientation: [f32; 2], // dx, dy
    pub is_hiding: bool,
    pub is_in_truck: bool,
    pub stamina: f32,
    pub is_running: bool,
    pub frame: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerGearState {
    pub player_id: NetworkId,
    pub left_hand: Option<NetworkId>,
    pub right_hand: Option<NetworkId>,
    pub inventory: Vec<NetworkId>,
    pub held_item: Option<NetworkId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GhostState {
    pub id: NetworkId,
    pub position: [f32; 3],
    pub warp: f32,
    pub hunt_warning_active: bool,
    pub hunt_warning_intensity: f32,
    pub calm_time_secs: f32,
    pub repellent_hits_delta: f32,
    pub repellent_misses_delta: f32,
    pub freezing_temp_clarity: f32,
    pub floating_orbs_clarity: f32,
    pub uv_ectoplasm_clarity: f32,
    pub emf_level5_clarity: f32,
    pub evp_recording_clarity: f32,
    pub spirit_box_clarity: f32,
    pub rl_presence_clarity: f32,
    pub cpm500_clarity: f32,
    pub visual_alpha_multiplier: f32,
    pub rage_tendency_multiplier: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapTileState {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub tileset: String,
    pub tileuid: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomSync {
    pub name: String,
    pub state: u8, // 0=Off, 1=On
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HauntedObjectSync {
    pub original_position: [i32; 3],
    pub tileset: String,
    pub tileuid: u32,
    pub influence_type: InfluenceType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GearSyncState {
    pub id: NetworkId,
    pub kind: ungear_core::types::gear::kind::GearKind,
    pub position: [f32; 3],
    pub is_on: bool,
    pub is_deployed: bool,
    pub deployed_direction: [f32; 2],
    pub details: GearDetails,
    pub battery: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissionResult {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
    pub base_score: i64,
    pub difficulty_multiplier: f32,
    pub grade_multiplier: f64,
    pub average_sanity: f32,
    pub player_count: u32,
    pub alive_count: u32,
    pub full_score: i64,
    pub mission_successful: bool,
    pub money_earned: i64,
    pub grade_achieved: Grade,
    pub required_deposit: i64,
    pub mission_reward_base: i64,
    pub deposit_originally_held: i64,
    pub deposit_returned_to_bank: i64,
    pub costs_deducted_from_deposit: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Message)]
pub enum TransientEvent {
    PlaySound {
        sound_file: String,
        volume: f32,
        position: Option<[f32; 3]>,
    },
    SpawnParticle {
        particle_type: String,
        position: [f32; 3],
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    /// Initial handshake from Client to Host.
    Hello {
        version: String,
        previous_id: Option<NetworkId>,
    },
    /// Response from Host to Client.
    Welcome {
        id: NetworkId,
        map_seed: u64,
        map_filepath: String,
        difficulty_id: String,
    },
    /// Periodic state update from Host to Client.
    Snapshot {
        tick: u64,
        is_full_sync: bool,
        app_state: AppState,
        game_state: GameState,
        players: Vec<PlayerState>,
        ghosts: Vec<GhostState>,
        rooms: Vec<RoomSync>,
        map_tiles: Vec<MapTileState>,
        gear: Vec<GearSyncState>,
        player_gear: Vec<PlayerGearState>,
        events: Vec<TransientEvent>,
        evidences_found: Vec<Evidence>,
        evidences_missing: Vec<Evidence>,
        ghost_type_guess: Option<GhostType>,
        ghosts_discarded: Vec<GhostType>,
        mission_result: Box<Option<MissionResult>>,
        repellent_crafted_count: u32,
        breach_position: Option<[f32; 3]>,
        ghost_type: Option<GhostType>,
        haunted_objects: Vec<HauntedObjectSync>,
    },
    /// Periodic input update from Client to Host.
    PlayerInput {
        player_id: NetworkId,
        movement: [f32; 2],
        run: bool,
        interact: bool,
        grab: bool,
        drop: bool,
        use_right_hand: bool,
        use_left_hand: bool,
        inventory_cycle: bool,
        inventory_swap: bool,
        target_position: Option<[f32; 2]>,
        aim_direction: [f32; 2],
    },
    /// Request to interact with a map tile.
    InteractionRequest {
        player_id: NetworkId,
        position: [i32; 3],
        interaction_type: InteractionExecutionType,
    },
    /// Host sends the final mission summary.
    MissionSummary { result: MissionResult },
    /// Client updates their journal guess.
    JournalUpdate {
        player_id: NetworkId,
        ghost_type: Option<GhostType>,
        evidences_found: Vec<Evidence>,
        evidences_missing: Vec<Evidence>,
    },
    /// Client requests to craft repellent.
    CraftRepellent {
        player_id: NetworkId,
        ghost_type: GhostType,
    },
    /// Client requests to enter the truck/van.
    RequestTruckEntry { player_id: NetworkId },
    /// Client requests to exit the truck/van.
    RequestTruckExit { player_id: NetworkId },
    /// A player has left the mission (ended their game or disconnected)
    PlayerLeft { player_id: NetworkId },
    /// Client requests a change in their truck loadout.
    RequestTruckInventoryChange {
        player_id: NetworkId,
        change: TruckInventoryChange,
    },
    /// Client requests to toggle an evidence state in the journal.
    RequestJournalEvidenceToggle {
        player_id: NetworkId,
        evidence: Evidence,
        discard: bool,
    },
    /// Client requests to toggle a ghost selection/discard in the journal.
    RequestJournalGhostToggle {
        player_id: NetworkId,
        ghost_type: GhostType,
        discard: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TruckInventoryChange {
    RemoveLeftHand,
    RemoveRightHand,
    RemoveInventoryIndex(usize),
    AddItem(ungear_core::types::gear::kind::GearKind),
}

#[derive(Debug, Clone, Message)]
pub struct NetworkDataEvent {
    pub message: NetworkMessage,
}

#[derive(Debug, Clone, Message)]
pub struct NetworkDisconnectEvent {
    pub id: NetworkId,
}

#[derive(Debug, Clone, Message)]
pub struct SendNetworkMessage(pub NetworkMessage);
