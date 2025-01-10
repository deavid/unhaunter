use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GameplaySettings {
    pub movement_style: MovementStyle,
    pub camera_controls: CameraControls,
    pub character_controls: CharacterControls,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum MovementStyle {
    #[default]
    Isometric,
    ScreenSpaceOrthogonal,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum CameraControls {
    #[default]
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum CharacterControls {
    #[default]
    WASD,
    Arrows,
}
