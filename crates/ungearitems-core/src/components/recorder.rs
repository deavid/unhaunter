use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct Recorder {
    pub frame_counter: u32,
    pub display_secs_since_last_update: f32,
    pub sound: f32,
    pub sound_l: Vec<f32>,
    pub amt_recorded: f32,
    pub evp_recorded_time_secs: f32,
    pub evp_recorded_display: bool,
    pub evp_recorded_count: usize,
    pub display_glitch_timer: f32, // Added for EMI effects
    pub false_reading_timer: f32,  // For creating false audio spikes
    pub blinking_hint_active: bool,
}
