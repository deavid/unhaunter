// TODO: The following two functions are logic and should not be exported from a -plugin crate.
// They are currently breaking the Core vs. Plugin architectural structure.
pub use lighting::rebuild_lighting_field;
pub use prebake::prebake_lighting_field;

pub mod plugin;

mod audio;
mod lighting;
mod maplight;
mod metrics;
mod prebake;
mod resources;
mod systems;
mod utils;
