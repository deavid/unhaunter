use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct GhostAssets {
    #[asset(path = "img/ghost.png")]
    pub ghost: Handle<Image>,
    #[asset(path = "img/breach.png")]
    pub breach: Handle<Image>,
    #[asset(path = "img/focus_ring_vignette.png")]
    pub focus_ring_vignette: Handle<Image>,
    #[asset(path = "img/miasma-base-01.png")]
    pub miasma: Handle<Image>,
}

pub const GHOST_BREACH_ANCHOR: Vec2 = Vec2::new(0.0, -0.3673469);
