use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use std::time::Duration;
use uncampaign_plugin::plugin::UnhaunterCampaignPlugin;
use unclassic_mode_plugin::plugin::ClassicModePlugin;
use unfog_plugin::plugin::UnhaunterFogPlugin;
use ungame_plugin::plugin::UnhaunterGamePlugin;
use ungear_plugin::plugin::UnhaunterGearPlugin;
use ungearitems_plugin::plugin::UnhaunterGearItemsPlugin;
use unghost_plugin::plugin::UnhaunterGhostPlugin;
use unlight_plugin::plugin::UnhaunterLightPlugin;
use unmainmenu_plugin::plugin::UnhaunterMenuPlugin;
use unmanual_plugin::plugin::UnhaunterManualPlugin;
use unmaphub_plugin::plugin::UnhaunterMapHubPlugin;
use unmapload_plugin::plugin::UnhaunterMapLoadPlugin;
use unmenu_plugin::plugin::UnhaunterCoreMenuPlugin;
use unmenusettings_plugin::plugin::UnhaunterMenuSettingsPlugin;
use unmetrics_plugin::plugin::UnmetricsPlugin;
use unmission_plugin::MissionPlugin;
use unnpc_plugin::plugin::UnhaunterNPCPlugin;
use unpicking_plugin::plugin::CustomSpritePickingPlugin;
use unplayer_plugin::plugin::UnhaunterPlayerPlugin;
use unprofile_plugin::plugin::UnhaunterProfilePlugin;
use unrender_plugin::plugin::UnhaunterRenderPlugin;
use unroot_plugin::plugin::UnhaunterRootPlugin;
use unsettings_plugin::plugin::UnhaunterSettingsPlugin;
use unsound_plugin::plugin::SoundPlugin;
use unsummary_plugin::plugin::UnhaunterSummaryPlugin;
use unthermal_plugin::plugin::ThermalPlugin;
use untmxmap_plugin::plugin::UnhaunterTmxMapPlugin;
use untruck_plugin::plugin::UnhaunterTruckPlugin;
use untypes_core::cli::CliOptions;
use untypes_core::platform::plt;
use unwalkie_plugin::plugin::UnhaunterWalkiePlugin;

pub fn app_run(cli_options: CliOptions) {
    let mut app = App::new();
    app.insert_resource(cli_options);
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("Unhaunter {}", plt::VERSION),
                    resolution: default_resolution(),
                    // Enabling VSync might make it easier in WASM? (It doesn't)
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::INFO,
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.04, 0.08, 0.14)))
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 15.0,
    )));

    app.add_plugins(FrameTimeDiagnosticsPlugin::new(1024));
    // app.add_plugins(LogDiagnosticsPlugin::default());

    // Add picking support for our custom sprites
    app.add_plugins(CustomSpritePickingPlugin);

    app.add_plugins((
        UnhaunterRootPlugin,
        UnmetricsPlugin,
        ThermalPlugin,
        SoundPlugin,
        UnhaunterRenderPlugin,
        UnhaunterManualPlugin,
        UnhaunterSummaryPlugin,
        UnhaunterGearPlugin,
        MissionPlugin,
    ));
    app.add_plugins((
        UnhaunterGearItemsPlugin,
        UnhaunterMapHubPlugin,
        UnhaunterTruckPlugin,
        UnhaunterGamePlugin,
        UnhaunterPlayerPlugin,
        UnhaunterGhostPlugin,
        UnhaunterMenuPlugin,
        UnhaunterLightPlugin,
        UnhaunterNPCPlugin,
    ));
    app.add_plugins((
        UnhaunterTmxMapPlugin,
        UnhaunterSettingsPlugin,
        UnhaunterMenuSettingsPlugin,
        UnhaunterFogPlugin,
        UnhaunterWalkiePlugin,
        UnhaunterCoreMenuPlugin,
        UnhaunterMapLoadPlugin,
        ClassicModePlugin,
        UnhaunterCampaignPlugin,
        UnhaunterProfilePlugin,
    ));
    app.run();
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width as u32, height as u32)
}
