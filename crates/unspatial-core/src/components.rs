use crate::boardposition::BoardPosition;
use bevy::prelude::*;

/// Records the original map position of a movable entity for network correlation
/// FIXME: This is a very bad idea and should be eventually removed. Additionally this is a unnet-core concept.
#[derive(Component, Debug, Clone)]
pub struct NetworkOriginalMapPosition {
    pub position: BoardPosition,
    pub tileset: String,
    pub tileuid: u32,
}
