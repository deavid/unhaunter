use bevy::prelude::*;
use uncore::assets::index::{AssetIdx, AssetIdxLoader};
use uncore::assets::tmxmap::{TmxMap, TmxMapLoader};
use uncore::assets::tsxsheet::{TsxSheet, TsxSheetLoader};
use unstd::tiledmap::MapTileSetDb;

use crate::init_maps::{MapAssetIndexHandle, init_maps, map_index_preload, tmxmap_preload};
use crate::load_level::load_level_handler;

pub struct UnhaunterTmxMapPlugin;

impl Plugin for UnhaunterTmxMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapTileSetDb>()
            .init_resource::<MapAssetIndexHandle>()
            .init_asset::<TmxMap>()
            .init_asset::<TsxSheet>()
            .init_asset::<AssetIdx>()
            .init_asset_loader::<TmxMapLoader>()
            .init_asset_loader::<TsxSheetLoader>()
            .init_asset_loader::<AssetIdxLoader>()
            .add_systems(Startup, init_maps)
            .add_systems(
                Update,
                (
                    map_index_preload,
                    tmxmap_preload,
                    load_level_handler,
                ),
            );
    }
}
