use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub(crate) struct CampaignAssets {
    #[asset(path = "img/badges.png")]
    pub badges: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 5, rows = 1))]
    pub badges_layout: Handle<TextureAtlasLayout>,
}
