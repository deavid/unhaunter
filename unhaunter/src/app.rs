use crate::app_args::AppArgs;
use bevy::ecs::schedule::ExecutorKind;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::{app::ScheduleRunnerPlugin, diagnostic::FrameTimeDiagnosticsPlugin};
use std::time::Duration;
use uncommon_app_core::platform::plt;

// Core & Logic Plugins
use unboard_plugin::plugin::UnhaunterBoardPlugin;
use uncampaign_plugin::plugin::UnhaunterCampaignPlugin;
use undifficulty_plugin::plugin::UnhaunterDifficultyPlugin;
use unhub_plugin::plugin::{UnhaunterHubCorePlugin, UnhaunterHubPlugin};
use uninteraction_plugin::plugin::{
    UnhaunterInteractionClientPlugin, UnhaunterInteractionCorePlugin,
};
use unmapload_plugin::plugin::{UnhaunterMapLoadCorePlugin, UnhaunterMapLoadPlugin};
use unmission_plugin::plugin::UnhaunterMissionPlugin;
use unreplicon_plugin::plugin::UnrepliconPlugin;
use unsettings_plugin::plugin::UnhaunterSettingsPlugin;
use unsoundfield_plugin::plugin::UnhaunterSoundFieldPlugin;
use untmxmap_plugin::plugin::UnhaunterTmxMapPlugin;

// Domain Logic (Common for all modes)
use ungear_plugin::plugin::{UnhaunterGearCorePlugin, UnhaunterGearPlugin};
use ungearitems_plugin::plugin::{
    UnhaunterGearItemsCorePlugin, UnhaunterGearItemsPresentationPlugin,
};
use unghost_logic::plugin::UnhaunterGhostLogicPlugin;
use unghost_presentation::plugin::GhostPresentationPlugin;
use unhaunter_gearitems_logic::plugin::UnhaunterGearItemsLogicPlugin;
use uninventory_plugin::plugin::UnhaunterInventoryPlugin;
use unlight_plugin::plugin::UnhaunterLightCorePlugin;
use unlight_presentation::plugin::UnlightPresentationPlugin;
use unlocomotion_plugin::plugin::UnhaunterLocomotionPlugin;
use unnavigation_plugin::plugin::{UnhaunterNavigationClientPlugin, UnhaunterNavigationPlugin};
use unnpc_plugin::plugin::{UnhaunterNPCCorePlugin, UnhaunterNPCPlugin};
use unplayer_plugin::plugin::{UnhaunterPlayerCorePlugin, UnhaunterPlayerPlugin};
use untruck_plugin::plugin::{UnhaunterTruckCorePlugin, UnhaunterTruckPlugin};
use untruckui_plugin::plugin::UnhaunterTruckUIPlugin;
use unvitals_plugin::plugin::UnhaunterVitalsPlugin;

// Rendering & Graphics
use unfog_plugin::plugin::{UnhaunterFogCorePlugin, UnhaunterFogPlugin};
use unpicking_plugin::plugin::CustomSpritePickingPlugin;
use unrender_plugin::plugin::{UnhaunterRenderCorePlugin, UnhaunterRenderPlugin};
use unspatial_plugin::plugin::UnhaunterSpatialPlugin;
use unthermal_plugin::plugin::UnhaunterThermalPlugin;

// Audio & Spatial
use unaudiobg_plugin::plugin::UnhaunterAudioBgPlugin;
use unaudiospatial_plugin::plugin::UnhaunterSpatialAudioPlugin;
use unwalkie_logic::plugin::UnhaunterWalkieLogicPlugin;
use unwalkie_plugin::plugin::{UnhaunterWalkieCorePlugin, UnhaunterWalkiePlugin};

// UI & Menu Systems
use unlobby_plugin::plugin::UnhaunterLobbyPlugin;
use unmainmenu_plugin::plugin::UnhaunterMenuPlugin;
use unmanual_plugin::plugin::UnhaunterManualPlugin;
use unmenu_plugin::plugin::UnhaunterCoreMenuPlugin;
use unmenusettings_plugin::plugin::UnhaunterMenuSettingsPlugin;
use unpause_plugin::plugin::UnpausePlugin;
use unsummary_plugin::plugin::{UnhaunterSummaryCorePlugin, UnhaunterSummaryPlugin};

// Input & Interaction
use uninput_plugin::plugin::UnhaunterInputPlugin;

// Gameplay Modes
use unclassic_mode_gameplay_plugin::plugin::ClassicModeGameplayPlugin;
use unclassic_mode_orchestrator_plugin::plugin::ClassicModeOrchestratorPlugin;
use unclassic_mode_render_plugin::plugin::ClassicModeRenderPlugin;
use unclassic_mode_ui_plugin::plugin::ClassicModeUiPlugin;

// Utilities & Diagnostics
use uncareer_plugin::plugin::UnhaunterCareerPlugin;
use unfps_plugin::plugin::UnhaunterFpsPlugin;
use unmaphub_plugin::plugin::UnhaunterMapHubPlugin;
use unmetrics_plugin::plugin::UnhaunterMetricsPlugin;
use unprofile_plugin::plugin::UnhaunterProfilePlugin;

use uncommon_states_core::{BootState, UIContextState};
use unmission_core::types::SimulationState;

pub fn app_build(args: AppArgs) -> App {
    let AppArgs {
        verbose,
        mute,
        include_draft_maps,
        net_mode,
        installation_id_file,
        dedicated,
        procman_channel,
        hub_url,
        cert_file,
        key_file,
        skip_ssl_verification,
    } = args;
    let mut app = App::new();

    let filter = crate::log_filter::build_log_filter(verbose);

    if dedicated {
        app.add_plugins((
            MinimalPlugins
                .set(ScheduleRunnerPlugin::run_loop(Duration::from_micros(
                    1_000_000 / 60,
                    // 1_000_000 / 10,
                )))
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(1),
                }),
            bevy::log::LogPlugin {
                level: bevy::log::Level::TRACE,
                filter,
                ..default()
            },
            bevy::asset::AssetPlugin::default(),
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::state::app::StatesPlugin,
            bevy::transform::TransformPlugin,
        ));
        // Force all continuous schedules to run systems one-by-one.
        // This reduces overhead on dedicated servers where threading is restricted.
        app.edit_schedule(First, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(PreUpdate, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(Update, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(SpawnScene, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(PostUpdate, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(Last, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(FixedFirst, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(FixedPreUpdate, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(FixedPostUpdate, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .edit_schedule(FixedLast, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        });
    } else {
        let mut default_plugins = DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Unhaunter {}", plt::VERSION),
                resolution: default_resolution(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        });

        if mute {
            info!("Audio muted via command line flag.");
            default_plugins = default_plugins.set(bevy::audio::AudioPlugin {
                global_volume: bevy::audio::GlobalVolume {
                    volume: bevy::audio::Volume::Linear(0.0),
                },
                ..default()
            });
        }

        app.add_plugins(default_plugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::TRACE,
            filter,
            ..default()
        }));

        app.add_plugins((
            FrameTimeDiagnosticsPlugin::new(1024),
            CustomSpritePickingPlugin,
        ));
    }

    app.insert_resource(ClearColor(Color::srgb(0.04, 0.08, 0.14)))
        .init_state::<UIContextState>()
        .init_state::<BootState>()
        .init_state::<SimulationState>()
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
            1.0 / 15.0,
        )));

    // == CORE & FOUNDATION ==
    app.add_plugins((
        UnhaunterSettingsPlugin,
        UnhaunterDifficultyPlugin,
        UnhaunterBoardPlugin,
        UnrepliconPlugin {
            role_config: unreplicon_plugin::systems::roles::RoleConfig {
                dedicated,
                intent: match &net_mode {
                    crate::app_args::CliNetMode::Offline => {
                        unreplicon_plugin::systems::roles::NetworkRoleIntent::Standalone
                    }
                    crate::app_args::CliNetMode::PeerHost { .. } => {
                        unreplicon_plugin::systems::roles::NetworkRoleIntent::Host
                    }
                    crate::app_args::CliNetMode::Join { .. } => {
                        unreplicon_plugin::systems::roles::NetworkRoleIntent::Client
                    }
                },
            },
            transport_config: match net_mode.clone() {
                crate::app_args::CliNetMode::Offline => {
                    unreplicon_transport::resources::TransportConfig::Offline
                }
                crate::app_args::CliNetMode::PeerHost {
                    port,
                    bind_addresses,
                } => unreplicon_transport::resources::TransportConfig::PeerHost {
                    port,
                    bind_addresses,
                    cert_file: cert_file.clone(),
                    key_file: key_file.clone(),
                    skip_ssl_verification,
                },
                crate::app_args::CliNetMode::Join { address, ticket } => {
                    unreplicon_transport::resources::TransportConfig::Join {
                        address,
                        ticket,
                        skip_ssl_verification,
                    }
                }
            },
            procman_config: unreplicon_transport::resources::ProcManConfig {
                procman_channel: procman_channel.clone(),
                port: match &net_mode {
                    crate::app_args::CliNetMode::PeerHost { port, .. } => *port,
                    _ => 0,
                },
                cert_file: cert_file.clone(),
                key_file: key_file.clone(),
                skip_ssl_verification,
            },
        },
        UnhaunterLobbyPlugin,
        UnhaunterTmxMapPlugin { include_draft_maps },
        UnhaunterMapLoadCorePlugin,
        UnhaunterMissionPlugin,
        UnhaunterHubCorePlugin,
        UnhaunterSoundFieldPlugin,
        UnhaunterMetricsPlugin,
        UnhaunterSummaryCorePlugin,
    ));

    // == DOMAIN LOGIC (Part 1: Player & Movement) ==
    app.add_plugins((
        UnhaunterPlayerCorePlugin,
        UnhaunterLocomotionPlugin,
        UnhaunterNavigationPlugin,
        UnhaunterVitalsPlugin,
        UnhaunterInventoryPlugin,
        UnhaunterGearCorePlugin,
        UnhaunterGearItemsCorePlugin,
        UnhaunterGearItemsLogicPlugin,
        UnhaunterGhostLogicPlugin,
        UnhaunterNPCCorePlugin,
        UnhaunterTruckCorePlugin,
        UnhaunterLightCorePlugin,
        UnhaunterInteractionCorePlugin,
        UnhaunterRenderCorePlugin,
    ));

    // == CAREER & ECONOMY ==
    app.add_plugins(UnhaunterCareerPlugin);

    // == WALKIE NET (all peers: registers message channels + server-side systems) ==
    app.add_plugins(UnhaunterWalkieLogicPlugin);

    // == DOMAIN LOGIC (Part 2: Gameplay Modes) ==
    app.add_plugins((ClassicModeOrchestratorPlugin, ClassicModeGameplayPlugin));
    app.add_plugins(UnhaunterSpatialAudioPlugin { enable: !dedicated });
    // == CLIENT-ONLY PLUGINS ==
    if !dedicated {
        // Input & Foundation
        app.add_plugins((
            UnhaunterInputPlugin,
            UnhaunterInteractionClientPlugin,
            UnhaunterNavigationClientPlugin,
            UnpausePlugin,
            UnhaunterPlayerPlugin,
            UnhaunterMenuPlugin,
            UnhaunterCoreMenuPlugin,
            UnhaunterMenuSettingsPlugin,
            UnhaunterManualPlugin,
            UnhaunterSummaryPlugin,
            UnhaunterTruckPlugin,
            UnhaunterTruckUIPlugin,
            UnhaunterHubPlugin {
                hub_url: hub_url.clone(),
            },
            UnhaunterNPCPlugin,
            UnhaunterWalkiePlugin,
        ));

        // Graphics & Rendering
        app.add_plugins((
            GhostPresentationPlugin,
            UnhaunterThermalPlugin,
            UnlightPresentationPlugin,
            UnhaunterFogPlugin,
            UnhaunterFogCorePlugin,
            UnhaunterSpatialPlugin,
            UnhaunterRenderPlugin,
            UnhaunterGearPlugin,
            UnhaunterGearItemsPresentationPlugin,
        ));

        // Audio
        app.add_plugins((UnhaunterAudioBgPlugin, UnhaunterWalkieCorePlugin));

        // Map & Campaign
        app.add_plugins((
            UnhaunterMapHubPlugin,
            UnhaunterCampaignPlugin,
            UnhaunterMapLoadPlugin,
        ));

        // UI for Gameplay Modes
        app.add_plugins((ClassicModeRenderPlugin, ClassicModeUiPlugin));

        // Diagnostics
        app.add_plugins((
            UnhaunterFpsPlugin,
            UnhaunterProfilePlugin {
                installation_id_file: installation_id_file.clone(),
            },
        ));
    }

    app
}

pub fn app_run(args: AppArgs) {
    let mut app = app_build(args);
    app.run();
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width as u32, height as u32)
}
