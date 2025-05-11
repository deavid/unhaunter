pub mod data;
pub mod plugin;

mod dev_tools;

use bevy::prelude::*;
use dev_tools::{snapshot_schema_system, validate_schema_snapshots};
use uncore::states::AppState;

pub struct UnprofilePlugin;

impl Plugin for UnprofilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, validate_schema_snapshots);
        app.add_systems(OnExit(AppState::Summary), snapshot_schema_system);
    }
}

// Re-export key types for easier access from other crates
pub use data::PlayerProfileData;
