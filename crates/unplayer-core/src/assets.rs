use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct PlayerAssets {
    #[asset(path = "img/characters-model1-demo.png")]
    pub character: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 64, columns = 16, rows = 4))]
    pub character_layout: Handle<TextureAtlasLayout>,
}

pub const PLAYER_ANCHOR: Vec2 = Vec2::new(0.0, -0.3958333); // calc(13, 43, 26, 48)
