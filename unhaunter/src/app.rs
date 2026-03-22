use bevy::ecs::schedule::ExecutorKind;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::{app::ScheduleRunnerPlugin, diagnostic::FrameTimeDiagnosticsPlugin};
use std::time::Duration;
use untypes_core::cli::CliOptions;
use untypes_core::platform::plt;

// Core & Logic Plugins
use unboard_plugin::plugin::UnhaunterBoardPlugin;
use uncampaign_plugin::plugin::UnhaunterCampaignPlugin;
use undifficulty_plugin::plugin::UnhaunterDifficultyPlugin;
use unhub_plugin::plugin::UnhaunterHubPlugin;
use uninteraction_plugin::plugin::UnhaunterInteractionCorePlugin;
use unmapload_plugin::plugin::UnhaunterMapLoadPlugin;
use unmission_plugin::plugin::UnhaunterMissionPlugin;
use unreplicon_plugin::plugin::UnrepliconPlugin;
use unsettings_plugin::plugin::UnhaunterSettingsPlugin;
use unsoundfield_plugin::plugin::UnhaunterSoundFieldPlugin;
use untmxmap_plugin::plugin::UnhaunterTmxMapPlugin;

// Domain Logic (Common for all modes)
use ungear_plugin::plugin::{UnhaunterGearCorePlugin, UnhaunterGearPlugin};
use ungearitems_plugin::plugin::{UnhaunterGearItemsCorePlugin, UnhaunterGearItemsPlugin};
use unghost_plugin::plugin::{UnhaunterGhostCorePlugin, UnhaunterGhostPlugin};
use uninventory_plugin::plugin::UnhaunterInventoryPlugin;
use unlight_plugin::plugin::{UnhaunterLightCorePlugin, UnhaunterLightPlugin};
use unlocomotion_plugin::plugin::UnhaunterLocomotionPlugin;
use unnavigation_plugin::plugin::UnhaunterNavigationPlugin;
use unnpc_plugin::plugin::{UnhaunterNPCCorePlugin, UnhaunterNPCPlugin};
use unplayer_plugin::plugin::{UnhaunterPlayerCorePlugin, UnhaunterPlayerPlugin};
use untruck_plugin::plugin::{UnhaunterTruckCorePlugin, UnhaunterTruckPlugin};
use unvitals_plugin::plugin::UnhaunterVitalsPlugin;

// Rendering & Graphics
use unfog_plugin::plugin::{UnhaunterFogCorePlugin, UnhaunterFogPlugin};
use unpicking_plugin::plugin::CustomSpritePickingPlugin;
use unrender_plugin::plugin::{UnhaunterRenderCorePlugin, UnhaunterRenderPlugin};
use unthermal_plugin::plugin::UnhaunterThermalPlugin;

// Audio & Spatial
use unaudiobg_plugin::plugin::UnhaunterAudioBgPlugin;
use unaudiospatial_plugin::plugin::UnhaunterSpatialAudioPlugin;
use unwalkie_plugin::plugin::{UnhaunterWalkieCorePlugin, UnhaunterWalkiePlugin};

// UI & Menu Systems
use unlobby_plugin::plugin::UnhaunterLobbyPlugin;
use unmainmenu_plugin::plugin::UnhaunterMenuPlugin;
use unmanual_plugin::plugin::UnhaunterManualPlugin;
use unmenu_plugin::plugin::UnhaunterCoreMenuPlugin;
use unmenusettings_plugin::plugin::UnhaunterMenuSettingsPlugin;
use unsummary_plugin::plugin::{UnhaunterSummaryCorePlugin, UnhaunterSummaryPlugin};
use unui_plugin::plugin::UnhaunterUiPlugin;

// Input & Interaction
use uninput_plugin::plugin::UnhaunterInputPlugin;

// Gameplay Modes
use unclassic_mode_gameplay_plugin::plugin::ClassicModeGameplayPlugin;
use unclassic_mode_orchestrator_plugin::plugin::ClassicModeOrchestratorPlugin;
use unclassic_mode_render_plugin::plugin::ClassicModeRenderPlugin;
use unclassic_mode_ui_plugin::plugin::ClassicModeUiPlugin;

// Utilities & Diagnostics
use unfps_plugin::plugin::UnhaunterFpsPlugin;
use unmaphub_plugin::plugin::UnhaunterMapHubPlugin;
use unmetrics_plugin::plugin::UnhaunterMetricsPlugin;
use unprofile_plugin::plugin::UnhaunterProfilePlugin;

pub fn app_run(cli_options: CliOptions) {
    let mut app = App::new();

    let filter = crate::log_filter::build_log_filter(cli_options.verbose);

    app.insert_resource(cli_options.clone());

    if cli_options.dedicated {
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

        if cli_options.mute {
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
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
            1.0 / 15.0,
        )));

    // == CORE & FOUNDATION ==
    app.add_plugins((
        UnhaunterSettingsPlugin,
        UnhaunterDifficultyPlugin,
        UnhaunterBoardPlugin,
        UnrepliconPlugin,
        UnhaunterLobbyPlugin,
        UnhaunterTmxMapPlugin,
        UnhaunterMapLoadPlugin,
        UnhaunterMissionPlugin,
        UnhaunterHubPlugin,
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
        UnhaunterGhostCorePlugin,
        UnhaunterNPCCorePlugin,
        UnhaunterTruckCorePlugin,
        UnhaunterLightCorePlugin,
        UnhaunterInteractionCorePlugin,
        UnhaunterRenderCorePlugin,
    ));

    // == DOMAIN LOGIC (Part 2: Gameplay Modes) ==
    app.add_plugins((ClassicModeOrchestratorPlugin, ClassicModeGameplayPlugin));

    // == CLIENT-ONLY PLUGINS ==
    if !cli_options.dedicated {
        // Input & Foundation
        app.add_plugins((
            UnhaunterInputPlugin,
            UnhaunterUiPlugin,
            UnhaunterPlayerPlugin,
            UnhaunterMenuPlugin,
            UnhaunterCoreMenuPlugin,
            UnhaunterMenuSettingsPlugin,
            UnhaunterManualPlugin,
            UnhaunterSummaryPlugin,
            UnhaunterTruckPlugin,
            UnhaunterNPCPlugin,
            UnhaunterWalkiePlugin,
        ));

        // Graphics & Rendering
        app.add_plugins((
            UnhaunterGhostPlugin,
            UnhaunterThermalPlugin,
            UnhaunterLightPlugin,
            UnhaunterFogPlugin,
            UnhaunterFogCorePlugin,
            UnhaunterRenderPlugin,
            UnhaunterGearPlugin,
            UnhaunterGearItemsPlugin,
        ));

        // Audio
        app.add_plugins((
            UnhaunterSpatialAudioPlugin,
            UnhaunterAudioBgPlugin,
            UnhaunterWalkieCorePlugin,
        ));

        // Map & Campaign
        app.add_plugins((UnhaunterMapHubPlugin, UnhaunterCampaignPlugin));

        // UI for Gameplay Modes
        app.add_plugins((ClassicModeRenderPlugin, ClassicModeUiPlugin));

        // Diagnostics
        app.add_plugins((UnhaunterFpsPlugin, UnhaunterProfilePlugin));
    }

    app.run();
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width as u32, height as u32)
}
