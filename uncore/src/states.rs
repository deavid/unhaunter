use bevy::prelude::*;

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Summary,
    MapHub,
    UserManual,
    PreplayManual,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    None,
    Truck,
    Pause,
    NpcHelp,
}
