use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq,
)]
pub struct GameplaySettings {
    pub movement_style: MovementStyle,
    pub camera_controls: CameraControls,
    pub character_controls: CharacterControls,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum GameplaySettingsValue {
    movement_style(MovementStyle),
    camera_controls(CameraControls),
    character_controls(CharacterControls),
}

#[derive(
    Reflect,
    Component,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::Display,
    strum::EnumIter,
)]
pub enum MovementStyle {
    #[default]
    Isometric,
    ScreenSpaceOrthogonal,
}

#[derive(
    Reflect,
    Component,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::Display,
    strum::EnumIter,
)]
pub enum CameraControls {
    #[default]
    On,
    Off,
}

#[derive(
    Reflect,
    Component,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Sequence,
    strum::Display,
    strum::EnumIter,
)]
pub enum CharacterControls {
    #[default]
    WASD,
    Arrows,
}
