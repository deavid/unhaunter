use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct GeigerCounter {
    pub display_secs_since_last_update: f32,
    pub frame_counter: u16,
    pub sound_a1: f32,
    pub sound_a2: f32,
    pub sound_display: f32, // Used for the display value
    pub sound_l: Vec<f32>,
    pub last_sound_time_secs: f32,
    pub output_sound: f32,
    pub blinking_hint_active: bool,
}
