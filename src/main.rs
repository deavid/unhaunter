mod behavior;
mod board;
mod game;
mod gear;
mod ghosts;
mod leveleditor;
mod mainmenu;
mod materials;
mod pause;
mod root;
mod tiledmap;
mod truck;

use std::time::Duration;

use bevy::{
    // diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
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
    //        .add_plugins(FrameTimeDiagnosticsPlugin)
    //        .add_plugins(LogDiagnosticsPlugin::default())
    //        .add_plugins(bevy_framepace::debug::DiagnosticsPlugin)
    .add_plugins(Material2dPlugin::<CustomMaterial1>::default())
    .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default())
    .add_plugins(bevy_framepace::FramepacePlugin)
    .add_systems(Startup, set_fps_limiter)
    .insert_resource(ClearColor(Color::rgb(0.04, 0.08, 0.14)))
    .init_resource::<tiledmap::MapTileSetDb>()
    .init_resource::<board::BoardData>()
    .init_resource::<board::SpriteDB>()
    .init_resource::<board::RoomDB>()
    .add_event::<board::BoardDataToRebuild>()
    .add_state::<root::State>()
    .add_state::<root::GameState>()
    .add_event::<mainmenu::MenuEvent>()
    .add_systems(Startup, root::load_assets)
    .add_systems(OnEnter(root::State::MainMenu), mainmenu::setup)
    .add_systems(OnExit(root::State::MainMenu), mainmenu::cleanup)
    .add_systems(Update, mainmenu::setup_ui)
    .add_systems(Update, mainmenu::keyboard)
    .add_systems(Update, mainmenu::item_logic)
    .add_systems(Update, mainmenu::menu_event)
    .add_systems(Update, gear::update_gear_inventory)
    .add_systems(Update, gear::thermometer::temperature_update)
    .add_systems(Update, gear::recorder::sound_update)
    .add_systems(
        Update,
        gear::keyboard_gear.run_if(in_state(root::GameState::None)),
    )
    .add_systems(Update, board::apply_perspective)
    .add_systems(PostUpdate, board::boardfield_update)
    .add_systems(Update, leveleditor::apply_lighting)
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 30.0,
    )));
    game::app_setup(&mut app);
    truck::app_setup(&mut app);

    app.run();
}
