use bevy::prelude::*;
use unassets_core::assets::index::{AssetIdx, AssetIdxLoader};
use unassets_core::assets::tmxmap::{TmxMap, TmxMapLoader};
use unassets_core::assets::tsxsheet::{TsxSheet, TsxSheetLoader};
use untiled_core::MapTileSetDb;

use crate::init_maps::MapAssetIndexHandle;

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
            .init_asset_loader::<AssetIdxLoader>();

        crate::init_maps::app_setup(app);
        crate::load_level::app_setup(app);
    }
}
