use bevy::prelude::*;

/// Resource to know basic stuff of the game.
#[derive(Debug, Resource)]
pub struct GameConfig {
    /// Which player should the camera and lighting follow
    pub player_id: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { player_id: 1 }
    }
}
