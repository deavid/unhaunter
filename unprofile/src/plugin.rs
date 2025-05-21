use crate::data::PlayerProfileData;
use bevy::prelude::*;
use bevy_persistent::prelude::*;
use std::path::Path;

pub struct UnhaunterProfilePlugin;

impl Plugin for UnhaunterProfilePlugin {
    fn build(&self, app: &mut App) {
        let config_dir_path = dirs::config_dir()
            .map(|native_config_dir| native_config_dir.join("unhaunter-game").join("config"))
            .unwrap_or_else(|| {
                warn!("Could not find native config directory. Using local 'config/' directory.");
                Path::new("local").join("config")
            });

        let player_profile_persistence = Persistent::<PlayerProfileData>::builder()
            .name("player_profile")
            .format(StorageFormat::RonPrettyWithStructNames)
            .path(config_dir_path.join("player_profile.ron"))
            .default(PlayerProfileData::default())
            .build()
            .unwrap_or_else(|e| {
                panic!(
                    "CRITICAL: Failed to initialize player profile persistence setup: {:?}",
                    e
                )
            });

        app.insert_resource(player_profile_persistence);

        #[cfg(debug_assertions)]
        {
            #[cfg(target_os = "linux")]
            {
                use crate::dev_tools;
                app.add_systems(Startup, dev_tools::snapshot_schema_system)
                    .add_systems(Startup, dev_tools::validate_schema_snapshots);
            }
        }

        // Make sure any stuck deposit is properly set on startup
        app.add_systems(Startup, recover_stuck_insurance_deposit);
    }
}

fn recover_stuck_insurance_deposit(mut player_profile: ResMut<Persistent<PlayerProfileData>>) {
    if player_profile.progression.insurance_deposit > 0 {
        info!(
            "Recovering insurance deposit ({} Bank) due to incomplete previous session.",
            player_profile.progression.insurance_deposit
        );

        player_profile.progression.bank += player_profile.progression.insurance_deposit;
        player_profile.progression.insurance_deposit = 0;

        if let Err(e) = player_profile.persist() {
            panic!("Failed to persist PlayerProfileData: {:?}", e);
        }
    }
}
