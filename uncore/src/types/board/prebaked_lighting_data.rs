//! Prebaked lighting data for optimized light propagation.
//!
//! This module defines structures to store precomputed lighting information,
//! allowing for significant performance improvements by reusing calculations
//! for static elements in the map.
use bevy::utils::HashMap;

/// Holds precomputed lighting propagation data for a single tile.
///
/// This structure stores information about light propagation from this tile
/// to its neighbors, allowing for efficient BFS-based light propagation that
/// starts from prebaked seed points and extends through the map.
#[derive(Clone, Debug, Default)]
pub struct PrebakedLightingData {
    /// Base light data for this tile
    pub light_info: LightInfo,

    /// Propagation data for 4-way connections
    /// Stores which directions should be followed when extending the light wave
    pub propagation_dirs: PropagationDirections,

    /// Contains the contribution from each light source that affects this tile
    /// Maps source ID -> light contribution amount
    pub source_contributions: HashMap<u32, SourceContribution>,

    /// Indicates if this is a continuation point (e.g., door)
    pub is_continuation_point: bool,

    /// Stores pending light propagations that would continue if this point were unblocked
    /// Only relevant for continuation points (doors, windows)
    pub pending_propagations: Vec<PendingPropagation>,
}

/// Stores the base light information for a tile
#[derive(Clone, Debug, Default)]
pub struct LightInfo {
    /// Is this tile a light source
    pub is_source: bool,

    /// Unique identifier for this light source (only valid if is_source is true)
    pub source_id: u32,

    /// Lux value for this tile
    pub lux: f32,

    /// Light color (r, g, b)
    pub color: (f32, f32, f32),

    /// Light transmissivity factor
    pub transmissivity: f32,
}

/// Information about how much a specific light source contributes to this tile
#[derive(Clone, Debug)]
pub struct SourceContribution {
    /// Amount of light (lux) contributed by this source
    pub lux: f32,

    /// Color of the light from this source
    pub color: (f32, f32, f32),

    /// Remaining propagation distance when this light reached the tile
    pub remaining_distance: f32,
}

impl Default for SourceContribution {
    fn default() -> Self {
        Self {
            lux: 0.0,
            color: (1.0, 1.0, 1.0),
            remaining_distance: 0.0,
        }
    }
}

/// Information about light propagation that would continue past a continuation point
#[derive(Clone, Debug)]
pub struct PendingPropagation {
    /// ID of the light source
    pub source_id: u32,

    /// Direction the light would propagate
    pub direction: PropagationDirection,

    /// Intensity of light when it reached this point
    pub incoming_lux: f32,

    /// Color of the light at this point
    pub color: (f32, f32, f32),

    /// Remaining propagation distance
    pub remaining_distance: f32,
}

/// Represents a propagation direction
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PropagationDirection {
    North,
    East,
    South,
    West,
}

/// Contains information about which directions light can propagate from this tile
/// Uses a compact representation for the 4 cardinal directions
#[derive(Clone, Debug, Default)]
pub struct PropagationDirections {
    /// Can light propagate north (+Y)
    pub north: bool,

    /// Can light propagate east (+X)
    pub east: bool,

    /// Can light propagate south (-Y)
    pub south: bool,

    /// Can light propagate west (-X)
    pub west: bool,
}
