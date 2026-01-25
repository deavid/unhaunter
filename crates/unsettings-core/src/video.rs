use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Component, Resource, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoSettings {
    pub window_size: WindowSize,
    pub aspect_ratio: AspectRatio,
    pub ui_scale: Scale,
    pub font_scale: Scale,
    pub max_upscale_factor: UpscaleFactorChoice,
}

#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum VideoSettingsValue {
    window_size(WindowSize),
    aspect_ratio(AspectRatio),
    ui_scale(Scale),
    font_scale(Scale),
    max_upscale_factor(UpscaleFactorChoice),
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
pub enum UpscaleFactorChoice {
    #[default]
    #[strum(to_string = "Native (1x)")]
    Native,
    #[strum(to_string = "2x")]
    Upscale2x,
    #[strum(to_string = "3x")]
    Upscale3x,
    #[strum(to_string = "4x")]
    Upscale4x,
    #[strum(to_string = "6x")]
    Upscale6x,
}

impl UpscaleFactorChoice {
    pub fn factor(&self) -> u32 {
        match self {
            UpscaleFactorChoice::Native => 1,
            UpscaleFactorChoice::Upscale2x => 2,
            UpscaleFactorChoice::Upscale3x => 3,
            UpscaleFactorChoice::Upscale4x => 4,
            UpscaleFactorChoice::Upscale6x => 6,
        }
    }
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            window_size: WindowSize::Medium,
            aspect_ratio: AspectRatio::Ar16_9,
            ui_scale: Scale::Scale100,
            font_scale: Scale::Scale100,
            max_upscale_factor: UpscaleFactorChoice::Upscale6x,
        }
    }
}

#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
)]
pub enum WindowSize {
    Small,
    #[default]
    Medium,
    Big,
}

#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
)]
pub enum AspectRatio {
    Ar4_3,
    #[default]
    Ar16_10,
    Ar16_9,
}
#[derive(
    Reflect, Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, Sequence,
)]
pub enum Scale {
    Scale080,
    Scale090,
    #[default]
    Scale100,
    Scale110,
    Scale120,
}
