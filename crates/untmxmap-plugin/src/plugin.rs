use bevy::prelude::*;
use untiled_core::tiled::MapTileSetDb;
use untmxmap_core::assets::index::{AssetIdx, AssetIdxLoader};
use untmxmap_core::assets::tmxmap::{TmxMap, TmxMapLoader};
use untmxmap_core::assets::tsxsheet::{TsxSheet, TsxSheetLoader};
use untmxmap_core::resources::maps::Maps;
use untmxmap_core::resources::upscale::UpscaleIndex;

use crate::init_maps::MapAssetIndexHandle;

pub struct UnhaunterTmxMapPlugin {
    pub include_draft_maps: bool,
}

impl Plugin for UnhaunterTmxMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(untmxmap_core::resources::config::TmxMapConfig {
            include_draft_maps: self.include_draft_maps,
        })
        .init_resource::<MapTileSetDb>()
        .init_resource::<MapAssetIndexHandle>()
        .init_resource::<UpscaleIndex>()
        .init_resource::<Maps>()
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
