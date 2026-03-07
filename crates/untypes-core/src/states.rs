use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    EngineBoot,
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
    /// UX loading screen shown while the Authority Node prepares the mission.
    /// The client enters this state when a mission is requested and exits to
    /// `AppState::InGame` once `SimulationState::Ready` is confirmed.
    MissionLoading,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BootState {
    #[default]
    Loading,
    Ready,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    Running,
    Truck,
    Pause,
    NpcHelp,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SimulationState {
    #[default]
    Unloaded, // No map loaded, all simulation systems should be idle
    Loading,  // Map geometry loaded, fields are being allocated
    Spawning, // All fields allocated, level content ready
    Ready,    // InGame + simulation is safe to run
    /// Mission has ended. Simulation ticks have stopped. Board entities are
    /// being despawned and arrays are being zeroed before returning to Unloaded.
    TearingDown,
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
