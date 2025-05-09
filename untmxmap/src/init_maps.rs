use bevy::prelude::*;
use uncore::assets::index::AssetIdx;
use uncore::assets::tmxmap::TmxMap;
use uncore::assets::tsxsheet::TsxSheet;
use uncore::types::root::map::Sheet;
use uncore::{resources::maps::Maps, types::root::map::Map};

pub struct PreLoad<A: Asset> {
    path: String,
    handle: Handle<A>,
    processed: bool,
}

#[derive(Resource, Default)]
pub struct MapAssetIndexHandle {
    tmxidx: Handle<AssetIdx>,
    tsxidx: Handle<AssetIdx>,
    idxprocessed: bool,
    maps: Vec<PreLoad<TmxMap>>,
    sheets: Vec<PreLoad<TsxSheet>>,
}

pub fn init_maps(asset_server: Res<AssetServer>, mut mapsidx: ResMut<MapAssetIndexHandle>) {
    mapsidx.tmxidx = asset_server.load("index/maps-tmx.assetidx");
    mapsidx.tsxidx = asset_server.load("index/maps-tsx.assetidx");
}

pub fn map_index_preload(
    asset_server: Res<AssetServer>,
    idx_assets: Res<Assets<AssetIdx>>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
) {
    if mapsidx.idxprocessed {
        return;
    }
    let Some(maps) = idx_assets.get(&mapsidx.tmxidx) else {
        return;
    };
    let Some(sheets) = idx_assets.get(&mapsidx.tsxidx) else {
        return;
    };
    for path in &maps.assets {
        let handle: Handle<TmxMap> = asset_server.load(path);
        let path = path.to_string();
        mapsidx.maps.push(PreLoad {
            handle,
            path,
            processed: false,
        });
    }
    for path in &sheets.assets {
        let handle: Handle<TsxSheet> = asset_server.load(path);
        let path = path.to_string();
        mapsidx.sheets.push(PreLoad {
            handle,
            path,
            processed: false,
        });
    }
    mapsidx.idxprocessed = true;
}

pub fn tmxmap_preload(
    mut maps: ResMut<Maps>,
    tmx_assets: Res<Assets<TmxMap>>,
    mut mapsidx: ResMut<MapAssetIndexHandle>,
) {
    let mut cleanup_needed = false;
    if !mapsidx.idxprocessed {
        return;
    }
    for mapload in &mut mapsidx.maps {
        if mapload.processed {
            continue;
        }
        let tmx = tmx_assets.get(&mapload.handle);
        if let Some(tmx) = tmx {
            mapload.processed = true;
            cleanup_needed = true;
            let path = mapload.path.clone();
            let classname = tmx.class.clone();
            let display_name = tmx.props.display_name.clone();

            if classname != "UnhaunterMap1" {
                warn!(
                    "Unrecognized Class {:?} for map {:?} (Should be 'UnhaunterMap1')",
                    classname, path
                );
                continue;
            }

            let default_name = format!("Unnamed ({})", path.replace("maps/", ""));
            let display_name = if display_name.is_empty() {
                default_name
            } else {
                display_name
            };
            info!("Found map {display_name:?} at path {path:?}");

            maps.maps.push(Map {
                name: display_name,
                path,
                handle: mapload.handle.clone(),
            });
        }
    }
    maps.maps.sort_by_key(|x| x.path.clone());
    for sheet in &mut mapsidx.sheets {
        if !sheet.processed {
            maps.sheets.push(Sheet {
                path: sheet.path.clone(),
                handle: sheet.handle.clone(),
            });
            sheet.processed = true;
            cleanup_needed = true;
        }
    }
    if cleanup_needed {
        mapsidx.maps.retain(|x| !x.processed);
        mapsidx.sheets.retain(|x| !x.processed);
    }
}
