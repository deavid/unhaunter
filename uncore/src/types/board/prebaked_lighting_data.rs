//! Prebaked lighting data for optimized light propagation.
//!
//! This module defines structures to store precomputed lighting information,
//! allowing for significant performance improvements by reusing calculations
//! for static elements in the map.

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
}

/// Stores the base light information for a tile
#[derive(Clone, Debug, Default)]
pub struct LightInfo {
    /// Is this tile a light source
    pub is_source: bool,

    /// Lux value for this tile
    pub lux: f32,

    /// Light color (r, g, b)
    pub color: (f32, f32, f32),

    /// Light transmissivity factor
    pub transmissivity: f32,
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
