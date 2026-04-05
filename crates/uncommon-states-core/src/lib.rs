use bevy::prelude::*;

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum UIContextState {
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
    MissionSelect,
    Hub,
    MissionLoading,
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BootState {
    #[default]
    Loading,
    Ready,
}
