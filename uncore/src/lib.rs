pub mod assets;
pub mod behavior;
pub mod colors;
pub mod components;
pub mod controlkeys;
pub mod difficulty;
pub mod events;
pub mod metric_recorder;
pub mod platform;
pub mod plugin;
pub mod random_seed;
pub mod resources;
pub mod states;
pub mod systemparam;
pub mod systems;
pub mod traits;
pub mod types;
pub mod utils;

/// Enables/disables debug logs related to the player.
pub const DEBUG_PLAYER: bool = false;

/// Zero degrees Celsius in Kelvin.
pub const KELVIN_OFFSET: f32 = 273.15;

/// Converts a temperature from Celsius to Kelvin.
#[inline]
pub fn celsius_to_kelvin(celsius: f32) -> f32 {
    celsius + KELVIN_OFFSET
}

/// Converts a temperature from Kelvin to Celsius.
#[inline]
pub fn kelvin_to_celsius(kelvin: f32) -> f32 {
    kelvin - KELVIN_OFFSET
}
