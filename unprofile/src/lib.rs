pub mod data;
pub mod plugin;

mod dev_tools;

use bevy::prelude::*;

pub struct UnprofilePlugin;

impl Plugin for UnprofilePlugin {
    fn build(&self, _app: &mut App) {}
}

// Re-export key types for easier access from other crates
pub use data::PlayerProfileData;
