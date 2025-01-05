use crate::bevy::bevy_load_map;
use bevy::prelude::*;
use uncore::{
    events::loadlevel::{LevelLoadedEvent, LoadLevelEvent},
    states::AppState,
};
use unstd::tiledmap::MapTileSetDb;

pub fn load_level_handler(
    mut ev: EventReader<LoadLevelEvent>,
    mut evw: EventWriter<LevelLoadedEvent>,
    asset_server: Res<AssetServer>,
    mut tilesetdb: ResMut<MapTileSetDb>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    let mut ev_iter = ev.read();
    let Some(load_event) = ev_iter.next() else {
        return;
    };
    let map_filepath = load_event.map_filepath.clone();
    warn!("Load Level: {map_filepath}");
    let (_map, layers) = bevy_load_map(
        &map_filepath,
        &asset_server,
        &mut texture_atlases,
        &mut tilesetdb,
    );
    app_next_state.set(AppState::InGame);
    evw.send(LevelLoadedEvent {
        map_filepath,
        layers,
    });
}
