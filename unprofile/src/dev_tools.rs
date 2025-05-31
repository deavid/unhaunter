#![allow(dead_code)]

use crate::data::PlayerProfileData;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use std::env;
use std::path::PathBuf;

/// Helper function to locate the fixture directory for schema snapshots.
/// Returns `Some(PathBuf)` if the directory exists, or `None` if it does not.
pub fn get_fixture_directory() -> Option<PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let fixture_dir = PathBuf::from(manifest_dir).join("tests/fixtures/player_profiles");

    if fixture_dir.exists() {
        Some(fixture_dir)
    } else {
        None
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Startup, snapshot_schema_system)
        .add_systems(Startup, validate_schema_snapshots);
}

/// System to create a versioned snapshot of the player profile schema.
fn snapshot_schema_system(_player_profile: Res<Persistent<PlayerProfileData>>) {
    #[cfg(all(debug_assertions, target_os = "linux"))]
    {
        use ron::ser::{PrettyConfig, to_string_pretty};
        use std::fs;
        if let Some(fixture_dir) = get_fixture_directory() {
            let version = env!("CARGO_PKG_VERSION");
            let snapshot_file = fixture_dir.join(format!("player_profile_v{}.ron", version));

            let pretty_config = PrettyConfig::new();
            match to_string_pretty(_player_profile.get(), pretty_config) {
                Ok(serialized) => {
                    if let Err(e) = fs::write(&snapshot_file, serialized) {
                        error!("Failed to write snapshot file {:?}: {:?}", snapshot_file, e);
                    } else {
                        info!("Snapshot created: {:?}", snapshot_file);
                    }
                }
                Err(e) => {
                    error!("Failed to serialize player profile: {:?}", e);
                }
            }
        } else {
            warn!("Fixture directory not found. Skipping schema snapshot.");
        }
    }
}

/// System to validate schema snapshots against the current `PlayerProfileData` definition.
fn validate_schema_snapshots() {
    #[cfg(all(debug_assertions, target_os = "linux"))]
    {
        use crate::data::PlayerProfileData;
        use ron::de::from_str;
        use std::fs;
        if let Some(fixture_dir) = get_fixture_directory() {
            if let Ok(entries) = fs::read_dir(&fixture_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("ron") {
                        match fs::read_to_string(&path) {
                            Ok(content) => match from_str::<PlayerProfileData>(&content) {
                                Ok(_) => {
                                    info!("Fixture passed validation: {:?}", path);
                                }
                                Err(e) => {
                                    panic!(
                                        "Schema validation failed for fixture {:?}: {:?}",
                                        path, e
                                    );
                                }
                            },
                            Err(e) => {
                                error!("Failed to read fixture file {:?}: {:?}", path, e);
                            }
                        }
                    }
                }
            } else {
                error!("Failed to read fixture directory: {:?}", fixture_dir);
            }
        } else {
            warn!("Fixture directory not found. Skipping schema validation.");
        }
    }
}
