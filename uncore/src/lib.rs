pub mod assets;
pub mod behavior;
pub mod colors;
pub mod components;
pub mod controlkeys;
pub mod difficulty;
pub mod events;
pub mod platform;
pub mod resources;
pub mod states;
pub mod systemparam;
pub mod systems;
pub mod traits;
pub mod types;
pub mod utils;

/// Enables the use of arrow keys for movement instead of WASD
pub const USE_ARROW_KEYS: bool = false;

/// Enables/disables debug logs related to the player.
pub const DEBUG_PLAYER: bool = false;
