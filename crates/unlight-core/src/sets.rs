use bevy::prelude::*;

/// System sets for ordering light gather and apply phases across crate boundaries.
///
/// `Gather` runs first and populates `ActiveFlashlights` and exposure.
/// `Apply` runs after and reads those resources to paint sprites/tiles.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LightUpdateSet {
    Gather,
    Apply,
}
