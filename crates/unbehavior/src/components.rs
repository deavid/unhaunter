use bevy::prelude::*;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::orientation::Orientation;

pub use crate::behavior::{Interactive, NpcHelpDialog};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Ground;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Collision;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Opaque;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct UVSurface;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Light;
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HeatEmitter;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Door;
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Stairs {
    pub z: i32,
}
#[derive(Component, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Reflect)]
#[reflect(Component)]
pub struct FloorItemCollidable;

/// Marker component for movable objects.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Movable;

/// Marker component for hiding spots.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HidingSpot;

/// Stable Tiled-space identifier for dynamic map entities.
///
/// Uniquely identifies an entity by the Tiled layer it came from and the tile's
/// raw Tiled coordinates. This is the anchor used by the client-side "Stitcher"
/// to find the local placeholder entity that corresponds to a server-replicated
/// dynamic entity.
///
/// `layer_idx` is the 0-based index of the Tiled layer in the enumerated
/// `tile_layers_iter()` in `unmapload-plugin/src/level_setup.rs`.
/// `x` and `y` are the raw `tile.pos.x` and `tile.pos.y` from the Tiled map,
/// **before** the coordinate transformation applied in `process_and_spawn_tile`.
#[derive(
    Component, Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Reflect,
)]
#[reflect(Component)]
pub struct TmxEntityId {
    pub layer_idx: usize,
    pub x: i32,
    pub y: i32,
}

/// Marker component that identifies entities that ghosts can interact with.
///
/// This component is automatically added to entities during map loading if they have:
/// - Door behavior (for DoorSlam, DoorCreak, Lock interactions)
/// - Switch/Light behavior (for Toggle interactions)
/// - Breaker behavior (for TripBreaker interactions)
/// - Object properties that enable ghost interaction (throwable, nudgeable, haunt_movable)
///
/// Using a marker component allows for fast ECS queries and avoids having to check
/// behavior properties every frame during ghost interaction selection.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct InteractableByGhost;

/// Component that links an entity (like a switch) to a specific room.
///
/// This acts as a spatial pointer. When an interaction occurs (e.g., flipping a switch),
/// the `room_delta` is added to the entity's board position to calculate a target coordinate.
/// This target coordinate is then looked up in the `RoomTopology` to identify the room name,
/// allowing the interaction to affect the state of that entire room (e.g., lights) via `RoomStateMap`.
///
/// - `room_delta`: Offset vector from the entity pos to a tile inside the target room.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct RoomStateDelta {
    pub room_delta: BoardPosition,
}

impl RoomStateDelta {
    pub fn new_for_room(orientation: &Orientation) -> Self {
        Self {
            room_delta: match orientation {
                Orientation::XAxis => BoardPosition { x: -1, y: 1, z: 0 },
                Orientation::YAxis => BoardPosition { x: -1, y: 1, z: 0 },
                Orientation::Both => BoardPosition::default(),
                Orientation::None => BoardPosition::default(),
            },
        }
    }

    pub fn with_opposite_side(orientation: &Orientation, opposite_side: bool) -> Self {
        if !opposite_side {
            return Self::new_for_room(orientation);
        }

        // If opposite_side is true, we'll use the opposite direction for the room delta
        Self {
            room_delta: match orientation {
                Orientation::XAxis => BoardPosition { x: -1, y: -1, z: 0 },
                Orientation::YAxis => BoardPosition { x: -1, y: -1, z: 0 },
                Orientation::Both => BoardPosition::default(),
                Orientation::None => BoardPosition::default(),
            },
        }
    }
}
