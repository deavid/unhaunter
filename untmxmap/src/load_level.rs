use crate::{bevy::bevy_load_map, map_loader::UnhaunterMapLoader};
use bevy::prelude::*;
use uncore::{
    assets::{tmxmap::TmxMap, tsxsheet::TsxSheet},
    events::loadlevel::{LevelLoadedEvent, LoadLevelEvent},
    resources::maps::Maps,
};
use unstd::tiledmap::MapTileSetDb;

fn load_level_handler(
    mut ev: EventReader<LoadLevelEvent>,
    mut evw: EventWriter<LevelLoadedEvent>,
    asset_server: Res<AssetServer>,
    mut tilesetdb: ResMut<MapTileSetDb>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    maps: Res<Maps>,
    tmx_assets: Res<Assets<TmxMap>>,
    tsx_assets: Res<Assets<TsxSheet>>,
) {
    let mut ev_iter = ev.read();
    let Some(load_event) = ev_iter.next() else {
        return;
    };
    let map_filepath = load_event.map_filepath.clone();
    warn!("Load Level: {map_filepath}");
    let tiled_map = UnhaunterMapLoader::load(&map_filepath, &maps, &tmx_assets, &tsx_assets);

    let (layers, floor_mapping) = bevy_load_map(
        tiled_map,
        &asset_server,
        &mut texture_atlases,
        &mut tilesetdb,
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
