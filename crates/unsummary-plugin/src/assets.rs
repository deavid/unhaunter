use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub(crate) struct SummaryAssets {
    #[asset(path = "img/title.png")]
    pub title: Handle<Image>,
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Light.ttf")]
    pub font_londrina_light: Handle<Font>,
}
