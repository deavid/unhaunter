use bevy::prelude::*;
use bevy_persistent::prelude::*;
use std::path::Path;
use unprofile_core::profile::{PlayerProfileData, RuntimeInstallationId};
use untypes_core::cli::CliOptions;
use uuid::Uuid;

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
                crate::dev_tools::app_setup(app);
            }
        }

        // Make sure any stuck deposit is properly set on startup
        app_setup(app);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Startup,
        (initialize_installation_id, recover_stuck_insurance_deposit),
    )
    .add_systems(
        Update,
        (
            crate::systems::apply_mission_completion_to_profile,
            crate::systems::record_death_to_profile,
        ),
    );
}

fn initialize_installation_id(
    mut commands: Commands,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    cli: Res<CliOptions>,
) {
    let mut installation_id = None;

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(path_str) = &cli.installation_id_file {
        let path = Path::new(path_str);
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let content = content.trim();
                match Uuid::parse_str(content) {
                    Ok(uuid) => {
                        if uuid.is_nil() {
                            eprintln!(
                                "ERROR: Override UUID from {} cannot be nil (all zeros).",
                                path_str
                            );
                            std::process::exit(1);
                        }
                        info!(
                            "Using installation ID override from file {}: {}",
                            path_str, uuid
                        );
                        installation_id = Some(uuid);
                    }
                    Err(e) => {
                        eprintln!(
                            "ERROR: Failed to parse UUID from installation-id-file {}: {:?}",
                            path_str, e
                        );
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "ERROR: Failed to read installation-id-file {}: {:?}",
                    path_str, e
                );
                std::process::exit(1);
            }
        }
    }

    if installation_id.is_none() {
        if player_profile.installation_id.is_nil() {
            // TODO: WASM support for generating UUIDs might need a different approach
            // for better entropy, but as multiplayer is not yet supported on WASM,
            // this is acceptable for now.
            let new_id = Uuid::new_v4();
            player_profile.installation_id = new_id;
            if let Err(e) = player_profile.persist() {
                error!(
                    "Failed to persist PlayerProfileData with new installation_id: {:?}",
                    e
                );
            }
            installation_id = Some(new_id);
        } else {
            installation_id = Some(player_profile.installation_id);
        }
    }

    #[cfg(target_arch = "wasm32")]
    if cli.installation_id_file.is_some() {
        warn!("installation-id-file is not supported on WASM");
    }

    commands.insert_resource(RuntimeInstallationId(installation_id.unwrap()));
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
