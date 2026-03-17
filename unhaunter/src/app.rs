use bevy::ecs::schedule::ExecutorKind;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::{app::ScheduleRunnerPlugin, diagnostic::FrameTimeDiagnosticsPlugin};
use std::time::Duration;
use unboard_plugin::plugin::UnhaunterBoardPlugin;
use uncampaign_plugin::plugin::UnhaunterCampaignPlugin;
use unclassic_mode_gameplay_plugin::plugin::ClassicModeGameplayPlugin;
use unclassic_mode_orchestrator_plugin::plugin::ClassicModeOrchestratorPlugin;
use unclassic_mode_render_plugin::plugin::ClassicModeRenderPlugin;
use unclassic_mode_ui_plugin::plugin::ClassicModeUiPlugin;
use undifficulty_plugin::plugin::UnhaunterDifficultyPlugin;
use unfog_plugin::plugin::{UnhaunterFogCorePlugin, UnhaunterFogPlugin};
use unfps_plugin::plugin::UnhaunterFpsPlugin;
use ungear_plugin::plugin::{UnhaunterGearCorePlugin, UnhaunterGearPlugin};
use ungearitems_plugin::plugin::{UnhaunterGearItemsCorePlugin, UnhaunterGearItemsPlugin};
use unghost_plugin::plugin::{UnhaunterGhostCorePlugin, UnhaunterGhostPlugin};
use unhub_plugin::plugin::UnhaunterHubPlugin;
use uninteraction_plugin::plugin::UnhaunterInteractionCorePlugin;
use unlight_plugin::plugin::{UnhaunterLightCorePlugin, UnhaunterLightPlugin};
use unlobby_plugin::plugin::UnhaunterLobbyPlugin;
use unmainmenu_plugin::plugin::UnhaunterMenuPlugin;
use unmanual_plugin::plugin::UnhaunterManualPlugin;
use unmaphub_plugin::plugin::UnhaunterMapHubPlugin;
use unmapload_plugin::plugin::UnhaunterMapLoadPlugin;
use unmenu_plugin::plugin::UnhaunterCoreMenuPlugin;
use unmenusettings_plugin::plugin::UnhaunterMenuSettingsPlugin;
use unmetrics_plugin::plugin::UnhaunterMetricsPlugin;
use unmission_plugin::plugin::UnhaunterMissionPlugin;
use unnpc_plugin::plugin::{UnhaunterNPCCorePlugin, UnhaunterNPCPlugin};
use unpicking_plugin::plugin::CustomSpritePickingPlugin;
use unplayer_plugin::plugin::{UnhaunterPlayerCorePlugin, UnhaunterPlayerPlugin};
use unprofile_plugin::plugin::UnhaunterProfilePlugin;
use unrender_plugin::plugin::{UnhaunterRenderCorePlugin, UnhaunterRenderPlugin};
use unreplicon_plugin::plugin::UnrepliconPlugin;
use unsettings_plugin::plugin::UnhaunterSettingsPlugin;
use unsound_plugin::plugin::UnhaunterSoundPlugin;
use unsummary_plugin::plugin::{UnhaunterSummaryCorePlugin, UnhaunterSummaryPlugin};
use unthermal_plugin::plugin::UnhaunterThermalPlugin;
use untmxmap_plugin::plugin::UnhaunterTmxMapPlugin;
use untruck_plugin::plugin::{UnhaunterTruckCorePlugin, UnhaunterTruckPlugin};
use untypes_core::cli::CliOptions;
use untypes_core::platform::plt;
use unui_plugin::plugin::UnhaunterUiPlugin;
use unwalkie_plugin::plugin::{UnhaunterWalkieCorePlugin, UnhaunterWalkiePlugin};

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

    // Common plugins (Logic, Core, Hardware-agnostic)
    app.add_plugins((
        UnhaunterSettingsPlugin,
        UnhaunterDifficultyPlugin,
        UnhaunterBoardPlugin,
        UnhaunterSummaryCorePlugin,
        UnhaunterMetricsPlugin,
        UnhaunterRenderCorePlugin,
        UnhaunterGearCorePlugin,
        UnhaunterInteractionCorePlugin,
        UnhaunterMissionPlugin,
        UnhaunterHubPlugin,
    ));
    app.add_plugins((
        UnhaunterTruckCorePlugin,
        UnhaunterPlayerCorePlugin,
        UnhaunterGhostCorePlugin,
        UnhaunterLightCorePlugin,
        UnhaunterNPCCorePlugin,
        UnrepliconPlugin,
        UnhaunterLobbyPlugin,
        UnhaunterTmxMapPlugin,
        UnhaunterMapLoadPlugin,
        ClassicModeOrchestratorPlugin,
        ClassicModeGameplayPlugin,
        UnhaunterGearItemsCorePlugin,
    ));

    // Client-side only plugins (UI, Graphics, Sound, Input)
    if !cli_options.dedicated {
        app.add_plugins((
            UnhaunterGearItemsPlugin,
            UnhaunterFpsPlugin,
            UnhaunterThermalPlugin,
            UnhaunterFogCorePlugin,
            UnhaunterWalkieCorePlugin,
            UnhaunterProfilePlugin,
        ));
        app.add_plugins((
            UnhaunterUiPlugin,
            UnhaunterManualPlugin,
            UnhaunterSummaryPlugin,
            UnhaunterPlayerPlugin,
            UnhaunterMenuPlugin,
            UnhaunterTruckPlugin,
            UnhaunterWalkiePlugin,
            UnhaunterNPCPlugin,
            UnhaunterMenuSettingsPlugin,
            UnhaunterCoreMenuPlugin,
            ClassicModeRenderPlugin,
            ClassicModeUiPlugin,
        ));
        app.add_plugins((
            UnhaunterGhostPlugin,
            UnhaunterSoundPlugin,
            UnhaunterLightPlugin,
            UnhaunterFogPlugin,
            UnhaunterRenderPlugin,
            UnhaunterGearPlugin,
            UnhaunterMapHubPlugin,
            UnhaunterCampaignPlugin,
        ));
    }

    app.run();
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width as u32, height as u32)
}
