use bevy::prelude::*;

/// Marks the entity whose position is used as the reference point for spatial audio
/// (distance-based volume and panning).
///
/// Exactly one entity should carry this component at any time. It is managed
/// exclusively by `unaudiospatial-plugin`, which inserts it whenever `MainPlayer`
/// is added to an entity and removes it when `MainPlayer` is removed.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct SpatialListener;
