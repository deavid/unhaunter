use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::window::WindowResolution;
use std::time::Duration;
use uncampaign::plugin::UnhaunterCampaignPlugin;
use uncore::difficulty::CurrentDifficulty;
use uncore::plugin::UnhaunterCorePlugin;
use uncore::{platform::plt, resources::object_interaction::ObjectInteractionConfig};
use uncoremenu::plugin::UnhaunterCoreMenuPlugin;
use unfog::plugin::UnhaunterFogPlugin;
use ungame::plugin::UnhaunterGamePlugin;
use ungear::plugin::UnhaunterGearPlugin;
use ungearitems::plugin::UnhaunterGearItemsPlugin;
use unghost::plugin::UnhaunterGhostPlugin;
use unlight::plugin::UnhaunterLightPlugin;
use unmaphub::plugin::UnhaunterMapHubPlugin;
use unmapload::plugin::UnhaunterMapLoadPlugin;
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
use unwalkie::plugin::UnhaunterWalkiePlugin;

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

    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
        .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());

    app.add_plugins((
        UnhaunterCorePlugin,
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
    ));
    app.add_plugins((
        UnhaunterTmxMapPlugin,
        UnhaunterSettingsPlugin,
        UnhaunterMenuSettingsPlugin,
        UnhaunterFogPlugin,
        UnhaunterWalkiePlugin,
        UnhaunterCoreMenuPlugin,
        UnhaunterMapLoadPlugin,
        UnhaunterCampaignPlugin,
    ));
    app.add_systems(Update, crate::report_timer::report_performance);
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_systems(Startup, set_window_icon);
    }
    app.run();
}

fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width, height)
}

#[cfg(not(target_arch = "wasm32"))]
use bevy::winit::WinitWindows;

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
) {
    // This only works on native. WASM uses the HTML icon.
    {
        use winit::window::Icon;
        let Some(assets_path) = crate::utils::find_assets_directory() else {
            warn!("Assets directory not found.");
            return;
        };
        // here we use the `image` crate to load our icon data from a png file
        // this is not a very bevy-native solution, but it will do
        let Ok(img) = image::open(assets_path.join("favicon-512x512.png")) else {
            warn!("Failed to load icon image.");
            return;
        };

        let (icon_rgba, icon_width, icon_height) = {
            let image = img.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

        // do it for all windows
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    }
}
