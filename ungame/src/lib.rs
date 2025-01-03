mod board;
pub mod difficulty;
mod game;
mod gear;
mod ghost;
mod ghost_definitions;
mod ghost_events;
pub mod ghost_setfinder;
mod mainmenu;
pub mod manual;
pub mod maphub;
mod maplight;
pub mod npchelp;
pub mod object_interaction;
mod pause;
mod player;
mod uncore_root;
pub mod root_plugin;
mod summary;
pub mod systems;
mod truck;
mod uncore_behavior;
mod uncore_materials;
mod uncore_tiledmap;

use uncore::utils; // FIXME: Delete this.

use uncore::platform::plt;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::Material2dPlugin,
    window::WindowResolution,
};
use object_interaction::ObjectInteractionConfig;
use std::time::Duration;
use uncore_materials::{CustomMaterial1, UIPanelMaterial};

const FPS_DEBUG: bool = false;

pub fn default_resolution() -> WindowResolution {
    let height = 800.0 * plt::UI_SCALE;
    let width = height * plt::ASPECT_RATIO;
    WindowResolution::new(width, height)
}

pub fn app_run() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("Unhaunter {}", plt::VERSION),
            resolution: default_resolution(),
            // Enabling VSync might make it easier in WASM? (It doesn't)
            present_mode: bevy::window::PresentMode::Fifo,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(Material2dPlugin::<CustomMaterial1>::default())
    .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default())
    .insert_resource(ClearColor(Color::srgb(0.04, 0.08, 0.14)))
    .init_resource::<uncore_tiledmap::MapTileSetDb>()
    .init_resource::<difficulty::CurrentDifficulty>()
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 15.0,
    )))
    .init_resource::<ObjectInteractionConfig>();
    if FPS_DEBUG {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(LogDiagnosticsPlugin::default());
    }
    arch_setup::app_setup(&mut app);
    root_plugin::app_setup(&mut app);
    gear::app_setup(&mut app);
    game::app_setup(&mut app);
    truck::app_setup(&mut app);
    summary::app_setup(&mut app);
    mainmenu::app_setup(&mut app);
    ghost::app_setup(&mut app);
    board::app_setup(&mut app);
    ghost_events::app_setup(&mut app);
    player::app_setup(&mut app);
    pause::app_setup(&mut app);
    maplight::app_setup(&mut app);
    npchelp::app_setup(&mut app);
    systems::object_charge::app_setup(&mut app);
    maphub::app_setup(&mut app);
    manual::app_setup(&mut app);
    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
mod arch_setup {
    use super::*;

    fn set_fps_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
        settings.limiter = bevy_framepace::Limiter::from_framerate(60.0);
    }

    pub fn app_setup(app: &mut App) {
        app.add_plugins(bevy_framepace::FramepacePlugin)
            .add_systems(Startup, set_fps_limiter);
        if FPS_DEBUG {
            app.add_plugins(bevy_framepace::debug::DiagnosticsPlugin);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod arch_setup {
    use super::*;

    pub fn app_setup(_app: &mut App) {}
}
