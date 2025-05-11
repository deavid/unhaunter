use bevy::prelude::*;

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    SettingsMenu,
    InGame,
    Summary,
    MapHub,
    UserManual,
    PreplayManual,
    MissionSelect, // Unified mission selection state for both Campaign and Custom missions
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    None,
    Truck,
    Pause,
    NpcHelp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum MapHubState {
    DifficultySelection,
    #[default]
    None,
}
