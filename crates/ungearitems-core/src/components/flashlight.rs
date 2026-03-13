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
}

impl Default for FlashlightSkin {
    fn default() -> Self {
        Self {
            inner_temp: Default::default(),
            heatsink_temp: Default::default(),
            frame_counter: Default::default(),
            rand: Default::default(),
            output_power: 0.0,
        }
    }
}

impl Flashlight {
    pub fn calculate_output_power(&self) -> f32 {
        match self.status {
            FlashlightStatus::Off => 0.0,
            FlashlightStatus::Low => 4.0,
            FlashlightStatus::Mid => 16.0,
            FlashlightStatus::High => 64.0,
        }
    }

    pub fn can_enable_status(&self, target_status: FlashlightStatus, battery_level: f32) -> bool {
        if target_status == FlashlightStatus::Off {
            return true; // Can always turn off
        }
        battery_level > 0.0
    }
}

impl FlashlightSkin {
    pub fn update_output_power(
        &mut self,
        status: &FlashlightStatus,
        battery_level: f32,
        glitch_timer: f32,
    ) {
        let mut new_power = match status {
            FlashlightStatus::Off => 0.0,
            FlashlightStatus::Low => 4.0,
            FlashlightStatus::Mid => 16.0,
            FlashlightStatus::High => 64.0,
        };
        if glitch_timer > 0.0 {
            new_power = glitch_timer * 4.0;
        }
        let bat = battery_level.sqrt() + 0.02;
        new_power *= bat;

        self.output_power = (self.output_power * 2.0 + new_power) / 3.0;
    }
}
