use bevy::prelude::*;
use uncommon_app_core::utils::temperature::celsius_to_kelvin;

#[derive(Component, Debug, Clone)]
pub struct Thermometer {
    pub temp: f32,
    pub temp_l2: [f32; 5],
    pub temp_l1: f32,
    pub frame_counter: u16,
    pub blinking_hint_active: bool,
}

impl Default for Thermometer {
    fn default() -> Self {
        Self {
            temp: celsius_to_kelvin(10.0),
            temp_l2: [celsius_to_kelvin(10.0); 5],
            temp_l1: celsius_to_kelvin(10.0),
            frame_counter: Default::default(),
            blinking_hint_active: false,
        }
    }
}
