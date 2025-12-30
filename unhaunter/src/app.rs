use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use bevy::window::WindowResolution;
use std::time::Duration;
use uncampaign_plugin::plugin::UnhaunterCampaignPlugin;
use uncore_assets::CliOptions;
use uncore_foundation::platform::plt;
use uncoremenu::plugin::UnhaunterCoreMenuPlugin;
use undifficulty_core::CurrentDifficulty;
use unfog_plugin::plugin::UnhaunterFogPlugin;
use ungame_plugin::plugin::UnhaunterGamePlugin;
use ungear::plugin::UnhaunterGearPlugin;
use ungearitems_plugin::plugin::UnhaunterGearItemsPlugin;
use unghost::plugin::UnhaunterGhostPlugin;
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;
use unlight_plugin::plugin::UnhaunterLightPlugin;
use unmanual_plugin::plugin::UnhaunterManualPlugin;
use unmaphub_plugin::plugin::UnhaunterMapHubPlugin;
use unmapload_plugin::plugin::UnhaunterMapLoadPlugin;
use unmenu_plugin::plugin::UnhaunterMenuPlugin;
use unmenusettings_plugin::plugin::UnhaunterMenuSettingsPlugin;
use unnpc_plugin::plugin::UnhaunterNPCPlugin;
use unpicking_plugin::CustomSpritePickingPlugin;
use unplayer::plugin::UnhaunterPlayerPlugin;
use unprofile_plugin::plugin::UnhaunterProfilePlugin;
use unrender::materials::{CustomMaterial1, UIPanelMaterial};
use unrender::plugin::UnhaunterBoardPlugin;
use unroot_plugin::UnhaunterRootPlugin;
use unsettings_plugin::plugin::UnhaunterSettingsPlugin;
use unsummary::plugin::UnhaunterSummaryPlugin;
use untmxmap_plugin::plugin::UnhaunterTmxMapPlugin;
use untruck_plugin::plugin::UnhaunterTruckPlugin;
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

    app.init_resource::<CurrentDifficulty>()
        .init_resource::<ObjectInteractionConfig>()
        .init_resource::<HauntState>();

    app.add_plugins(FrameTimeDiagnosticsPlugin::new(1024));
    // app.add_plugins(LogDiagnosticsPlugin::default());

    app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
        .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());

    // Add picking support for our custom sprites
    app.add_plugins(CustomSpritePickingPlugin);

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
        UnhaunterProfilePlugin,
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
    WindowResolution::new(width as u32, height as u32)
}

#[cfg(not(target_arch = "wasm32"))]
use bevy::ecs::system::NonSendMarker;

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(_marker: NonSendMarker, // Forces system to run on main thread
) {
    use bevy::winit::WINIT_WINDOWS;
    // This only works on native. WASM uses the HTML icon.
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

    // Access the thread-local static for window management
    WINIT_WINDOWS.with_borrow(|windows| {
        // do it for all windows
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    });
}
