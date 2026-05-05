use bevy::prelude::*;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Sequence,
    Display,
    EnumString,
    AsRefStr,
    Serialize,
    Deserialize,
    Reflect,
)]
pub enum FlashlightStatus {
    #[default]
    #[strum(serialize = "OFF")]
    Off,
    #[strum(serialize = "LOW")]
    Low,
    #[strum(serialize = "MID")]
    Mid,
    #[strum(serialize = " HI")]
    High,
}

/// The skeleton component for a flashlight, containing only replicated state.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default)]
pub struct Flashlight {
    pub status: FlashlightStatus,
}

/// The skin component for a flashlight, containing local simulation and visual state.
/// This component is never replicated.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct FlashlightSkin {
    pub inner_temp: f32,
    pub heatsink_temp: f32,
    pub frame_counter: u8,
    pub rand: u8,
    pub output_power: f32,
    /// Local flag to ensure the overheat sound only plays once per overheat event.
    pub overheat_sound_played: bool,
}

impl Default for FlashlightSkin {
    fn default() -> Self {
        Self {
            inner_temp: Default::default(),
            heatsink_temp: Default::default(),
            frame_counter: Default::default(),
            rand: Default::default(),
            output_power: 0.0,
            overheat_sound_played: false,
        }
    }
}
