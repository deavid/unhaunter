use bevy::prelude::*;
use enum_iterator::Sequence;
use strum::{AsRefStr, Display, EnumString};

#[derive(Debug, Clone, Default, PartialEq, Eq, Sequence, Display, EnumString, AsRefStr)]
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

#[derive(Component, Debug, Clone)]
pub struct Flashlight {
    pub status: FlashlightStatus,
    pub inner_temp: f32,
    pub heatsink_temp: f32,
    pub frame_counter: u8,
    pub rand: u8,
    pub output_power: f32,
}

impl Default for Flashlight {
    fn default() -> Self {
        Self {
            status: Default::default(),
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
    pub fn update_output_power(&mut self, battery_level: f32, glitch_timer: f32) {
        let mut new_power = self.calculate_output_power();
        if glitch_timer > 0.0 {
            new_power = glitch_timer * 4.0;
        }
        let bat = battery_level.sqrt() + 0.02;
        new_power *= bat;

        self.output_power = (self.output_power * 2.0 + new_power) / 3.0;
    }

    pub fn can_enable_status(&self, target_status: FlashlightStatus, battery_level: f32) -> bool {
        if target_status == FlashlightStatus::Off {
            return true; // Can always turn off
        }
        battery_level > 0.0 && self.inner_temp <= 1.0
    }
}
