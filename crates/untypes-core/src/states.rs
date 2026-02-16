use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    Lobby,
    SettingsMenu,
    InGame,
    Summary,
    MapHub,
    UserManual,
    PreplayManual,
    MissionSelect, // Unified mission selection state for both Campaign and Custom missions
    Hub,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    None,
    Truck,
    Pause,
    NpcHelp,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SimulationState {
    #[default]
    Inactive, // No map loaded, all simulation systems should be idle
    Initializing, // Map geometry loaded, fields are being allocated
    Ready,        // All fields allocated, level content ready
    Running,      // InGame + simulation is safe to run
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default, Serialize, Deserialize)]
pub enum MapHubState {
    DifficultySelection,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default, Serialize, Deserialize)]
pub enum LobbyScreen {
    #[default]
    None,
    Main,
    MapSelection,
    DifficultySelection,
}
