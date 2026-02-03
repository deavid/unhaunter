use bevy::prelude::*;
use unspatial_core::boardposition::BoardPosition;

/// Records the original map position of a movable entity for network correlation
#[derive(Component, Debug, Clone)]
pub struct OriginalMapPosition {
    pub position: BoardPosition,
    pub tileset: String,
    pub tileuid: u32,
}
