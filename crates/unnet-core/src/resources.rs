use crate::network_id::NetworkId;
use crate::messages::PlayerStatusInfo;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use untypes_core::states::AppState;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayer(pub Option<NetworkId>);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionEndRequested(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostGone(pub bool);

#[derive(Resource, Default, Debug, Clone)]
pub struct ChangedTiles(pub Vec<crate::messages::MapTileState>);

#[derive(Resource, Default, Debug, Clone)]
pub struct LobbyData {
    pub players: Vec<LobbyPlayer>,
    pub selected_map: Option<String>,
    pub selected_difficulty: Option<String>,
    pub host_app_state: Option<AppState>,
    pub mission_elapsed_secs: f32,
    pub evidences_found_count: u32,
    pub repellent_used: u32,
    pub player_statuses: Vec<PlayerStatusInfo>,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentMapSeed(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub id: NetworkId,
    pub tint_color_index: u8,
    pub connected: bool,
}
