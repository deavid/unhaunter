//! This module contains the main systems and plugins for managing the board in the Unhaunter game.
//! It includes systems for applying isometric perspective, rebuilding collision data, and updating
//! the lighting field based on the current state of the board and behaviors.

use bevy::prelude::*;

use unrender_std::board::spritedb::SpriteDB;
use unrender_std::custom_material1::CustomMaterial1;
use unrender_std::custom_material2::UIPanelMaterial;

use crate::metrics;

#[cfg(not(target_arch = "wasm32"))]
use bevy::ecs::system::NonSendMarker;

/// Core plugin for initializing board-related resources and simulation systems.
pub struct UnhaunterRenderCorePlugin;

impl Plugin for UnhaunterRenderCorePlugin {
    fn build(&self, app: &mut App) {
        let cli = app
            .world()
            .get_resource::<uncommon_app_core::cli::CliOptions>();
        let headless = cli.map(|c| c.dedicated).unwrap_or(false);

        crate::systems::lerp::app_setup(app);
        metrics::register_all(app);

        app.init_resource::<SpriteDB>();

        if headless {
            // In headless mode, register stub Assets<T> for the resources LoadLevelSystemParam requires
            app.init_asset::<CustomMaterial1>();
            app.init_asset::<Mesh>();
            app.init_asset::<Image>();
            app.init_asset::<TextureAtlasLayout>();
            app.init_asset::<bevy::audio::AudioSource>();
        }
    }
}

/// Plugin for initializing board-related visual systems and materials.
pub struct UnhaunterRenderPlugin;

impl Plugin for UnhaunterRenderPlugin {
    fn build(&self, app: &mut App) {
        let cli = app
            .world()
            .get_resource::<uncommon_app_core::cli::CliOptions>();
        let headless = cli.map(|c| c.dedicated).unwrap_or(false);

        if headless {
            return;
        }

        crate::systems::animation::app_setup(app);

        app.add_systems(
            Startup,
            unrender_std::resources::sprite_registry::setup_sprite_registry,
        );

        app.add_plugins(bevy::sprite_render::Material2dPlugin::<CustomMaterial1>::default())
            .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());

        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(Startup, set_window_icon);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(_marker: NonSendMarker, // Forces system to run on main thread
) {
    use bevy::winit::WINIT_WINDOWS;
    // This only works on native. WASM uses the HTML icon.
    use winit::window::Icon;
    let Some(assets_path) = uncommon_app_core::utils::find_assets::find_assets_directory() else {
        warn!("Assets directory not found.");
        return;
    };
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let Ok(img) = image::open(assets_path.join("favicon-512x512.png")) else {
        error!("Failed to load icon image.");
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
