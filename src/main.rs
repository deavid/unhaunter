mod behavior;
mod board;
mod game;
mod gear;
mod ghost;
mod ghost_definitions;
mod mainmenu;
mod maplight;
mod materials;
mod pause;
mod player;
mod root;
mod summary;
mod tiledmap;
mod truck;
mod utils;

use std::time::Duration;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::Material2dPlugin,
    window::WindowResolution,
};
use materials::{CustomMaterial1, UIPanelMaterial};

fn set_fps_limiter(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = bevy_framepace::Limiter::from_framerate(60.0);
    // bevy_framepace::debug::DiagnosticsPlugin
    //    bevy_framepace::FramePaceStats
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            // .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Unhaunter".to_string(),
                    resolution: WindowResolution::new(1500.0, 800.0),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(FrameTimeDiagnosticsPlugin)
    .add_plugins(LogDiagnosticsPlugin::default())
    .add_plugins(bevy_framepace::debug::DiagnosticsPlugin)
    .add_plugins(Material2dPlugin::<CustomMaterial1>::default())
    .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default())
    .add_plugins(bevy_framepace::FramepacePlugin)
    .add_systems(Startup, set_fps_limiter)
    .insert_resource(ClearColor(Color::rgb(0.04, 0.08, 0.14)))
    .init_resource::<tiledmap::MapTileSetDb>()
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 15.0,
    )));
    root::app_setup(&mut app);
    gear::app_setup(&mut app);
    game::app_setup(&mut app);
    truck::app_setup(&mut app);
    summary::app_setup(&mut app);
    mainmenu::app_setup(&mut app);
    ghost::app_setup(&mut app);
    board::app_setup(&mut app);
    player::app_setup(&mut app);
    pause::app_setup(&mut app);
    maplight::app_setup(&mut app);

    app.run();
}
