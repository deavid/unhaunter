use crate::{asset_index::AssetIdx, asset_tmxmap::TmxMap};
use bevy::prelude::*;
use uncore::{resources::maps::Maps, types::root::map::Map};

pub struct MapLoad {
    path: String,
    handle: Handle<TmxMap>,
    mapprocessed: bool,
}

#[derive(Resource, Default)]
pub struct MapAssetIndexHandle {
    mapsidx: Handle<AssetIdx>,
    idxprocessed: bool,
    maps: Vec<MapLoad>,
}

pub fn init_maps(
    asset_server: Res<AssetServer>,
    // maps: ResMut<Maps>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
) {
    // arch::init_maps(maps);
    mapsidx.mapsidx = asset_server.load("index/maps.assetidx");
}

pub fn map_index_preload(
    asset_server: Res<AssetServer>,
    idx_assets: Res<Assets<AssetIdx>>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
) {
    if mapsidx.idxprocessed {
        return;
    }
    let maps = idx_assets.get(&mapsidx.mapsidx);
    if let Some(maps) = maps {
        for path in &maps.assets {
            warn!("MAP INDEX LOADED: {path}");
            let handle: Handle<TmxMap> = asset_server.load(path);
            let path = path.to_string();
            mapsidx.maps.push(MapLoad {
                handle,
                path,
                mapprocessed: false,
            });
        }
        mapsidx.idxprocessed = true;
    }
}

pub fn tmxmap_preload(
    mut maps: ResMut<Maps>,
    tmx_assets: Res<Assets<TmxMap>>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
) {
    if !mapsidx.idxprocessed {
        return;
    }
    for mapload in &mut mapsidx.maps {
        if mapload.mapprocessed {
            continue;
        }
        let tmx = tmx_assets.get(&mapload.handle);
        if let Some(tmx) = tmx {
            mapload.mapprocessed = true;
            let path = mapload.path.clone();
            let classname = tmx.class.clone();
            let display_name = tmx.display_name.clone();

            if classname.is_none() {
                debug!("Ignored TMX {path:?} because it doesn't have a classname (Should be 'UnhaunterMap1')");
                continue;
            }

            if classname != Some("UnhaunterMap1".to_string()) {
                warn!(
                    "Unrecognized Class {:?} for map {:?} (Should be 'UnhaunterMap1')",
                    classname, path
                );
                continue;
            }

            let default_name = format!("Unnamed ({})", path.replace("maps/", ""));
            let display_name = display_name.unwrap_or(default_name);
            info!("Found map {display_name:?} at path {path:?}");
            maps.maps.push(Map {
                name: display_name,
                path,
            });
        }
    }
}
