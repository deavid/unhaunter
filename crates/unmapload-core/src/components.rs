// PendingTiledLayerProperties was removed from this module.
// Its replacement, PendingProperties (using the Tiled-free BehaviorProperties type),
// now lives in unbehavior_core::components::PendingProperties.

use bevy::prelude::Component;
use unbehavior_core::behavior::Behavior;

/// Core-owned visual identity for a map tile entity.
///
/// The mapload core emits this data so client-side render systems can hydrate
/// visual bundles without the core plugin depending on render assets.
#[derive(Component, Clone, Debug)]
pub struct TileVisualRef {
    pub tileset: String,
    pub tileuid: u32,
    pub flip_x: bool,
}

#[derive(Clone)]
pub struct MapTileComponents {
    pub behavior: Behavior,
}
