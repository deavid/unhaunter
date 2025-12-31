use bevy::prelude::*;

/// Alpha threshold for pixel-perfect picking (80%)
pub const ALPHA_THRESHOLD: f32 = 0.8;

/// An optional component that marks cameras that should be used for custom sprite picking.
///
/// Only needed if [`CustomSpritePickingSettings::require_markers`] is set to `true`.
#[derive(Debug, Clone, Default, Component)]
pub struct CustomSpritePickingCamera;

/// How the custom sprite picking backend should handle sprite boundaries.
#[derive(Debug, Clone, Copy, Default)]
pub enum CustomSpritePickingMode {
    /// Only consider the logical bounding box of sprites.
    ///
    /// This is the fastest method and works well for most use cases.
    /// The bounding box size is determined by [`CustomSpritePickingSettings::tile_size`].
    #[default]
    BoundingBox,
    /// Use pixel-perfect picking with transparency threshold.
    ///
    /// This mode samples the actual texture at the click point and checks if the alpha
    /// value is above the threshold (80%). This provides accurate hit detection that
    /// respects sprite transparency and actual sprite bounds.
    AlphaThreshold(f32),
}

/// Runtime settings for the custom sprite picking backend.
#[derive(Resource)]
pub struct CustomSpritePickingSettings {
    /// When `true`, only cameras marked with [`CustomSpritePickingCamera`] will perform picking.
    ///
    /// When `false` (default), all active cameras will be considered for picking.
    /// This is useful for fine-grained control in multi-camera setups.
    pub require_markers: bool,

    /// Determines how sprite boundaries are calculated for hit testing.
    pub picking_mode: CustomSpritePickingMode,

    /// Size of the clickable area around each sprite in world units.
    ///
    /// This defines the bounding box dimensions for hit testing.
    /// Default is 16x16 pixels, which works well for typical sprite sizes.
    /// Adjust based on your sprite dimensions and desired click tolerance.
    pub tile_size: Vec2,

    /// When `true`, enables checking neighboring pixels for more forgiving hit detection.
    ///
    /// If the exact pixel under the cursor doesn't meet the alpha threshold,
    /// the system will check nearby pixels. This makes small sprites easier to click
    /// and provides better user experience on touch devices.
    /// Only applies when using `CustomSpritePickingMode::AlphaThreshold`.
    pub check_neighbors: bool,

    /// When `true`, uses 8-way connectivity (including diagonals) for neighbor checking.
    ///
    /// When `false`, uses 4-way connectivity (cardinal directions only).
    /// 8-way provides more forgiving hit detection but checks more pixels.
    /// Only relevant when `check_neighbors` is `true`.
    pub use_8_way_neighbors: bool,

    /// Distance in pixels to check for neighboring pixels.
    ///
    /// A value of 1 checks immediate neighbors, 2 checks up to 2 pixels away, etc.
    /// Higher values provide more forgiving clicking but may affect precision.
    /// Only relevant when `check_neighbors` is `true`.
    pub neighbor_distance: u32,
}

impl Default for CustomSpritePickingSettings {
    fn default() -> Self {
        Self {
            require_markers: false,
            picking_mode: CustomSpritePickingMode::AlphaThreshold(ALPHA_THRESHOLD),
            tile_size: Vec2::new(16.0, 16.0),
            check_neighbors: true,
            use_8_way_neighbors: true,
            neighbor_distance: 1,
        }
    }
}
