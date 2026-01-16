use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct MapAssets {
    #[asset(path = "img/vignette.png")]
    pub vignette: Handle<Image>,
}

pub const GRID_1X1_ANCHOR: Vec2 = Vec2::new(0.0, -0.2045455); // calc(18, 31, 36, 44)
pub const GRID_1X1X4_ANCHOR: Vec2 = Vec2::new(0.0, -0.3673469); // calc(18, 85, 36, 98)
