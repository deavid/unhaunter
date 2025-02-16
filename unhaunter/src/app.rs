use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::window::WindowResolution;
use std::time::Duration;
use uncore::difficulty::CurrentDifficulty;
use uncore::{platform::plt, resources::object_interaction::ObjectInteractionConfig};
use unfog::plugin::UnhaunterFogPlugin;
use ungame::plugin::UnhaunterGamePlugin;
use ungear::plugin::UnhaunterGearPlugin;
use ungearitems::plugin::UnhaunterGearItemsPlugin;
use unghost::plugin::UnhaunterGhostPlugin;
use unlight::plugin::UnhaunterLightPlugin;
use unmaphub::plugin::UnhaunterMapHubPlugin;
use unmenu::plugin::UnhaunterMenuPlugin;
use unmenusettings::plugin::UnhaunterMenuSettingsPlugin;
use unnpc::plugin::UnhaunterNPCPlugin;
use unplayer::plugin::UnhaunterPlayerPlugin;
use unsettings::plugin::UnhaunterSettingsPlugin;
use unstd::materials::{CustomMaterial1, UIPanelMaterial};
use unstd::plugins::board::UnhaunterBoardPlugin;
use unstd::plugins::manual::UnhaunterManualPlugin;
use unstd::plugins::root::UnhaunterRootPlugin;
use unstd::plugins::summary::UnhaunterSummaryPlugin;
use untmxmap::plugin::UnhaunterTmxMapPlugin;
use untruck::plugin::UnhaunterTruckPlugin;

pub fn app_run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("Unhaunter {}", plt::VERSION),
            resolution: default_resolution(),
            // Enabling VSync might make it easier in WASM? (It doesn't)
            present_mode: bevy::window::PresentMode::AutoVsync,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.04, 0.08, 0.14)))
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 15.0,
    )));

    app.init_resource::<CurrentDifficulty>()
        .init_resource::<ObjectInteractionConfig>();

    // app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
        .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());

    app.add_plugins((
        UnhaunterRootPlugin,
        UnhaunterBoardPlugin,
        UnhaunterManualPlugin,
        UnhaunterSummaryPlugin,
        UnhaunterGearPlugin,
        UnhaunterGearItemsPlugin,
        UnhaunterMapHubPlugin,
        UnhaunterTruckPlugin,
        UnhaunterGamePlugin,
        UnhaunterPlayerPlugin,
        UnhaunterGhostPlugin,
        UnhaunterMenuPlugin,
        UnhaunterLightPlugin,
        UnhaunterNPCPlugin,
        UnhaunterTmxMapPlugin,
    ));
    app.add_plugins((
        UnhaunterSettingsPlugin,
        UnhaunterMenuSettingsPlugin,
        UnhaunterFogPlugin,
    ));
    // app.add_systems(Update, report_timer::report_performance);
    app.run();
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width, height)
}
