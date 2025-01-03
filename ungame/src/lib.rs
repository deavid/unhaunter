mod game;
mod ghost;
mod ghost_events;
mod mainmenu;
mod maphub;
mod maplight;
mod npchelp;
mod pause;
mod player;
mod systems;
mod truck;
mod uncore_board;
mod uncore_difficulty;
mod uncore_root;

use bevy::{prelude::*, sprite::Material2dPlugin, window::WindowResolution};
use std::time::Duration;
use uncore::utils;
use uncore::{platform::plt, resources::object_interaction::ObjectInteractionConfig};
use ungear::plugin::UnhaunterGearPlugin;
use ungearitems::plugin::UnhaunterGearItemsPlugin;
use unstd::materials::{CustomMaterial1, UIPanelMaterial};
use unstd::plugins::board::UnhaunterBoardPlugin;
use unstd::plugins::manual::UnhaunterManualPlugin;
use unstd::plugins::root::UnhaunterRootPlugin;
use unstd::plugins::summary::UnhaunterSummaryPlugin;
use unstd::tiledmap::bevy::MapTileSetDb;

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
    .insert_resource(ClearColor(Color::srgb(0.04, 0.08, 0.14)))
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
        1.0 / 15.0,
    )));

    app.init_resource::<MapTileSetDb>()
        .init_resource::<uncore_difficulty::CurrentDifficulty>()
        .init_resource::<ObjectInteractionConfig>();

    app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
        .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());

    app.add_plugins((
        UnhaunterRootPlugin,
        UnhaunterBoardPlugin,
        UnhaunterManualPlugin,
        UnhaunterSummaryPlugin,
        UnhaunterGearPlugin,
        UnhaunterGearItemsPlugin,
    ));

    game::app_setup(&mut app);
    truck::app_setup(&mut app);
    mainmenu::app_setup(&mut app);
    ghost::app_setup(&mut app);
    ghost_events::app_setup(&mut app);
    player::app_setup(&mut app);
    pause::app_setup(&mut app);
    maplight::app_setup(&mut app);
    npchelp::app_setup(&mut app);
    systems::object_charge::app_setup(&mut app);
    maphub::app_setup(&mut app);

    app.run();
}
