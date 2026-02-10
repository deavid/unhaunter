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
        liquid_content: Option<GhostType>,
    },
    SpiritBox {
        charge: f32,
        ghost_answer: bool,
    },
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: NetworkId,
    pub position: [f32; 3],
    pub orientation: [f32; 2], // dx, dy
    pub target_position: Option<[f32; 2]>,
    pub is_hiding: bool,
    pub is_in_truck: bool,
    pub is_spectating: bool,
    pub stamina: f32,
    pub health: f32,
    pub sanity: f32,
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
    pub hunt_target: bool,
    pub calm_time_secs: f32,
    pub repellent_hits_delta: f32,
    pub repellent_misses_delta: f32,
    pub repellent_hits: i64,
    pub class: GhostType,
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
    pub cvo_key: String,
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
pub struct MovableObjectSync {
    pub id: NetworkId,
    pub original_position: [i32; 3],
    pub tileset: String,
    pub tileuid: u32,
    pub current_position: [f32; 3],
    pub held_by: Option<NetworkId>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrabRequestMsg {
    pub player_id: NetworkId,
    pub target_id: NetworkId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrabResponseMsg {
    pub player_id: NetworkId,
    pub target_id: NetworkId,
    pub success: bool,
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
pub struct SnapshotMsg {
    pub tick: u64,
    pub is_full_sync: bool,
    pub app_state: AppState,
    pub game_state: GameState,
    pub can_end_mission: bool,
    pub players: Vec<PlayerState>,
    pub ghosts: Vec<GhostState>,
    pub rooms: Vec<RoomSync>,
    pub map_tiles: Vec<MapTileState>,
    pub gear: Vec<GearSyncState>,
    pub player_gear: Vec<PlayerGearState>,
    pub events: Vec<TransientEvent>,
    pub evidences_found: Vec<Evidence>,
    pub evidences_missing: Vec<Evidence>,
    pub ghost_type_guess: Option<GhostType>,
    pub ghosts_discarded: Vec<GhostType>,
    pub mission_result: Box<Option<MissionResult>>,
    pub repellent_crafted_count: u32,
    pub breach_position: Option<[f32; 3]>,
    pub ghost_type: Option<GhostType>,
    pub haunted_objects: Vec<HauntedObjectSync>,
    pub movable_objects: Vec<MovableObjectSync>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessage {
    /// Initial handshake from Client to Host.
    Hello {
        version: String,
        installation_id: uuid::Uuid,
    },
    /// Response from Host to Client.
    Welcome {
        id: NetworkId,
        map_seed: u64,
        map_filepath: String,
        difficulty_id: String,
    },
    /// Periodic state update from Host to Client.
    Snapshot(Box<SnapshotMsg>),
    /// Periodic input update from Client to Host.
    PlayerInput {
        player_id: NetworkId,
        o_position: Option<[f32; 3]>,
        movement: [f32; 2],
        run: bool,
        interact: bool,
        use_right_hand: bool,
        use_left_hand: bool,
        target_right_hand: Option<(bool, GearDetails)>,
        target_left_hand: Option<(bool, GearDetails)>,
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
    /// Client requests to hide.
    RequestHide { player_id: NetworkId },
    /// Client requests to unhide.
    RequestUnhide { player_id: NetworkId },
    /// Client requests to end the mission globally.
    RequestEndMission,
    /// A player has left the mission (ended their game or disconnected)
    PlayerLeft { player_id: NetworkId },
    /// Client requests a change in their truck loadout.
    RequestTruckInventoryChange {
        player_id: NetworkId,
        change: TruckInventoryChange,
    },
    /// Client requests to grab a movable object.
    GrabRequest(GrabRequestMsg),
    /// Host responds to a grab request.
    GrabResponse(GrabResponseMsg),
    /// Client requests to drop their currently held object.
    DropRequest { player_id: NetworkId },
    /// Client requests to cycle their inventory.
    CycleInventoryRequest { player_id: NetworkId },
    /// Client requests to swap their hands.
    SwapHandsRequest { player_id: NetworkId },
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
    /// Client requests a full state sync after map load.
    RequestFullSync { player_id: NetworkId },
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
    /// On the host, identifies which client sent this message.
    /// On the client (and for local passthrough messages), this is always None.
    pub source: Option<NetworkId>,
}

#[derive(Debug, Clone, Message)]
pub struct NetworkDisconnectEvent {
    pub id: NetworkId,
}

/// Emitted on the host when a client completes handshake and is ready to play.
#[derive(Debug, Clone, Message)]
pub struct PlayerJoinedEvent {
    pub id: NetworkId,
}

#[derive(Debug, Clone, Message)]
pub struct SendNetworkMessage(pub NetworkMessage);
