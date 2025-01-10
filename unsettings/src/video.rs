use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VideoSettings {
    pub window_size: WindowSize,
    pub aspect_ratio: AspectRatio,
    pub ui_scale: Scale,
    pub font_scale: Scale,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum WindowSize {
    Small,
    #[default]
    Medium,
    Big,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum AspectRatio {
    Ar4_3,
    #[default]
    Ar16_10,
    Ar16_9,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence)]
pub enum Scale {
    Scale080,
    Scale090,
    #[default]
    Scale100,
    Scale110,
    Scale120,
}
