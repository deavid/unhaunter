use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct GearAssets {
    #[asset(path = "img/gear_spritesheetA_48x48.png")]
    pub gear: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 96, columns = 10, rows = 10))]
    pub gear_layout: Handle<TextureAtlasLayout>,
}
