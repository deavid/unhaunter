use bevy::prelude::*;

use crate::{
    asset_index::{AssetIdx, AssetIdxLoader},
    asset_tmxmap::{TmxMap, TmxMapLoader},
    bevy::MapTileSetDb,
    init_maps::{init_maps, map_index_preload, tmxmap_preload, MapAssetIndexHandle},
};

pub struct UnhaunterTmxMapPlugin;

impl Plugin for UnhaunterTmxMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapTileSetDb>()
            .init_resource::<MapAssetIndexHandle>()
            .init_asset::<TmxMap>()
            .init_asset::<AssetIdx>()
            .init_asset_loader::<TmxMapLoader>()
            .init_asset_loader::<AssetIdxLoader>()
            .add_systems(Startup, init_maps)
            .add_systems(Update, (map_index_preload, tmxmap_preload));
    }
}
