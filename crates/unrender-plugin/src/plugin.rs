//! This module contains the main systems and plugins for managing the board in the Unhaunter game.
//! It includes systems for applying isometric perspective, rebuilding collision data, and updating
//! the lighting field based on the current state of the board and behaviors.

use bevy::diagnostic::{Diagnostic, DiagnosticPath, RegisterDiagnostic};
use bevy::prelude::*;

use unbehavior::roomdb::RoomDB;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unmetrics_core::metrics::SendMetric;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::perspective;
use unspatial_core::position::Position;

use unrender_std::board::spritedb::SpriteDB;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::materials::{CustomMaterial1, UIPanelMaterial};

#[cfg(not(target_arch = "wasm32"))]
use bevy::ecs::system::NonSendMarker;

pub const APPLY_PERSPECTIVE: DiagnosticPath =
    DiagnosticPath::const_new("unboard/systems/apply_perspective");

/// Main system of board that moves the tiles to their correct place on the screen
/// following the isometric perspective.
///
/// # Arguments
///
/// * `q` - A query for entities with `Position` and `Transform` components that have changed.
pub fn apply_perspective(
    mut q: Query<
        (&Position, &mut Transform, Option<&SpriteLayer>),
        Or<(Changed<Position>, Changed<SpriteLayer>)>,
    >,
) {
    let measure = APPLY_PERSPECTIVE.time_measure();

    for (pos, mut transform, layer) in q.iter_mut() {
        let mut translation = perspective::to_screen_coord(*pos);
        if let Some(layer) = layer {
            translation.z += layer.0;
        }
        transform.translation = translation;
    }

    measure.end_ms();
}

/// Plugin for initializing board-related resources and systems.
pub struct UnhaunterRenderPlugin;

impl Plugin for UnhaunterRenderPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::animation::app_setup(app);
        crate::systems::board_sync::app_setup(app);
        app.add_systems(
            Startup,
            unrender_std::resources::sprite_registry::setup_sprite_registry,
        );
        app.init_resource::<BoardTopology>()
            .init_resource::<BoardEntityField>()
            .init_resource::<BoardCollisionField>()
            .init_resource::<VisibilityData>()
            .init_resource::<SpriteDB>()
            .init_resource::<RoomDB>()
            .add_systems(Update, apply_perspective);
        app.register_diagnostic(Diagnostic::new(APPLY_PERSPECTIVE).with_suffix("ms"));

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
    let Some(assets_path) = unfoundation_core::utils::find_assets_directory() else {
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
