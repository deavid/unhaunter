use bevy::prelude::*;
use std::collections::HashMap;
pub use uuid::Uuid;

/// Server-side mapping from an active OwnerId to the
/// player's stable installation UUID (extracted from the JWT ticket at
/// connection time).
#[derive(Resource, Debug, Default)]
pub struct ClientUuidMap(pub HashMap<crate::ownership::OwnerId, Uuid>);

/// Identifies which player is the local (owning) player on this game instance.
///
/// `None` means the local player has not yet been assigned (e.g. in offline play
/// before a session starts, or in a dedicated server where there is no local player).
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayer(pub Option<Uuid>);

/// Accumulator flag: set to `true` once all players are back in the truck.
///
/// Gated by the mission-end logic; the truck UI reads it to enable the "End Mission" button.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionEndRequested(pub bool);

/// Set to `true` when the host has disconnected from the game.
///
/// Clients use this to detect a lost connection and display the appropriate UI.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostGone(pub bool);

/// The random seed used to generate the current mission's level.
///
/// All clients (and the server) share the same seed so the map is identical everywhere.
#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentMapSeed(pub u64);

/// Client-side flag indicating whether mission auto-join is currently armed.
///
/// When `true`, lobby UI can show deployment progress text and hide the manual
/// Join button. When `false`, clients should use the normal manual Join flow.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionAutoJoinArmed(pub bool);

/// Credentials returned by the hub server that identify this client's room.
///
/// `code` is a human-readable join code; `secret` is used for hub server auth.
#[derive(Resource, Default, Debug, Clone)]
pub struct RoomIdentification {
    pub code: Option<String>,
    pub secret: Option<String>,
}

/// Inserted by the client when ServerGamePhase::Concluding is observed.
/// Tracks the local fade-to-black timer before transitioning to Summary.
#[derive(Resource)]
pub struct MissionConcludingCinematic {
    /// Countdown timer. When finished, the client transitions to AppState::Summary
    /// provided SummaryData also exists.
    pub timer: Timer,
    /// Whether player inputs have been disabled for the duration of this cinematic.
    pub inputs_blocked: bool,
}

/// A single floor gear item currently present in the game world.
///
/// Stored in [`FloorGearCache`] on the server. Used to replay all existing floor
/// items to clients that join mid-mission.
#[derive(Debug, Clone)]
pub struct FloorGearEntry {
    pub kind: ungear_core::types::gear::kind::GearKind,
    /// World-space position `[x, y, z]`.
    pub pos: [f32; 3],
    /// Direction the gear is facing `[dx, dy, dz]`.
    pub direction: [f32; 3],
}

/// Server-only resource tracking all gear items currently on the floor.
///
/// Maintained by `broadcast_floor_gear_drop` (push) and `broadcast_floor_gear_pickup`
/// (pop nearest). Reset to empty on `OnEnter/OnExit(AppState::InGame)` so stale
/// items never carry over between missions.
///
/// When a new client connects mid-mission, `send_floor_gear_to_new_client` iterates
/// this cache and sends `FloorGearSpawnBroadcast` directly to the new client.
#[derive(Resource, Debug, Default)]
pub struct FloorGearCache {
    pub entries: Vec<FloorGearEntry>,
}
