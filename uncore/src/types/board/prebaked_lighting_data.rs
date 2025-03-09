//! Prebaked lighting data for optimized light propagation.
//!
//! This module defines structures to store precomputed lighting information,
//! allowing for significant performance improvements by reusing calculations
//! for static elements in the map.

use bevy::{ecs::entity::Entity, utils::HashMap};

use crate::components::board::boardposition::BoardPosition;

/// Holds precomputed lighting propagation data for a single tile.
///
/// This structure stores minimal information about light sources and wave edges,
/// supporting an efficient BFS-based light propagation algorithm that can be
/// resumed at runtime when dynamic elements (like doors) change state.
#[derive(Clone, Debug, Default)]
pub struct PrebakedLightingData {
    /// Base light data for this tile
    pub light_info: LightInfo,

    /// Indicates if this is a wave edge (point where light wave stopped propagating)
    /// These points can be used to continue light propagation at runtime
    /// for dynamic elements like doors
    pub wave_edge: Option<WaveEdge>,
}

#[derive(Clone, Debug, Default)]
pub struct WaveEdge {
    /// The lux intensity for the corresponding light at the source
    pub src_light_lux: f32,
    /// The distance travelled by the light wave
    pub distance_travelled: f32,
    /// Current position
    pub current_pos: (f32, f32, f32),
    /// IIR filtered mean position
    pub iir_mean_pos: (f32, f32, f32),
    /// IIR filtered mean of the iir_mean_pos
    pub iir_mean_iir_mean_pos: (f32, f32, f32),
}

/// Represents a wave edge tile with its associated data for light propagation.
#[derive(Debug, Clone)]
pub struct WaveEdgeData {
    /// The position of the wave edge on the board.
    pub position: BoardPosition,
    /// The ID of the light source associated with this wave edge.
    pub source_id: u32,
    /// The light intensity (lux) at the wave edge.
    pub lux: f32,
    /// The color of the light at the wave edge.
    pub color: (f32, f32, f32),
    /// The wave edge data containing propagation history.
    pub wave_edge: WaveEdge,
}

/// Stores the base light information for a tile
#[derive(Clone, Debug, Default)]
pub struct LightInfo {
    /// The light source ID, or None if not a source
    pub source_id: Option<u32>,

    /// Amount of lux intensity in this tile
    pub lux: f32,

    /// Light color (r, g, b)
    pub color: (f32, f32, f32),
}

/// Stores general metadata useful for speeding up light rebuilds.
#[derive(Clone, Debug, Default)]
pub struct PrebakedMetadata {
    pub light_sources: Vec<(Entity, (usize, usize, usize))>,
    pub doors: Vec<Entity>,
    pub light_source_ids: HashMap<Entity, u32>,
}
