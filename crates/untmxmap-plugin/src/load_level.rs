use crate::{bevy::bevy_load_map, map_loader::UnhaunterMapLoader};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unmapload_core::events::loadlevel::{LevelLoadedEvent, LoadLevelEvent};
use unsettings_core::video::VideoSettings;
use untiled_core::tiled::MapTileSetDb;
use untmxmap_core::assets::{tmxmap::TmxMap, tsxsheet::TsxSheet};
use untmxmap_core::resources::maps::Maps;
use untmxmap_core::resources::upscale::UpscaleIndex;

fn load_level_handler(
    mut ev: MessageReader<LoadLevelEvent>,
    mut evw: MessageWriter<LevelLoadedEvent>,
    asset_server: Res<AssetServer>,
    mut tilesetdb: ResMut<MapTileSetDb>,
    mut texture_atlases: Option<ResMut<Assets<TextureAtlasLayout>>>,
    maps: Res<Maps>,
    tmx_assets: Res<Assets<TmxMap>>,
    tsx_assets: Res<Assets<TsxSheet>>,
    upscale_idx: Res<UpscaleIndex>,
    video_settings: Res<Persistent<VideoSettings>>,
    cli: Res<untypes_core::cli::CliOptions>,
) {
    let mut ev_iter = ev.read();
    let Some(load_event) = ev_iter.next() else {
        return;
    };
    let map_filepath = load_event.map_filepath.clone();

    // F-04 patch: guard against missing map entries on headless server.
    // Full visual/logic separation is deferred to D-01 (F-18).
    if !maps.maps.iter().any(|m| m.path == map_filepath) {
        warn!(
            "load_level_handler: map '{}' not found in Maps resource; \
             skipping load (headless server with missing asset?)",
            map_filepath
        );
        return;
    }

    info!("Load Level: {map_filepath}");
    let tiled_map = UnhaunterMapLoader::load(&map_filepath, &maps, &tmx_assets, &tsx_assets);

    let (layers, floor_mapping) = bevy_load_map(
        tiled_map,
        &asset_server,
        &mut texture_atlases,
        &mut tilesetdb,
        &upscale_idx,
        &video_settings,
        cli.is_headless(),
    );

    evw.write(LevelLoadedEvent {
        map_filepath,
        layers,
        floor_mapping,
    });
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, load_level_handler);
}
