use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default, Serialize, Deserialize)]
pub enum LobbyScreen {
    #[default]
    None,
    Main,
    MapSelection,
    DifficultySelection,
}
