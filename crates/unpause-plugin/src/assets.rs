use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub(crate) struct PauseAssets {
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Light.ttf")]
    pub font_londrina_light: Handle<Font>,
    #[asset(path = "fonts/kode_mono/static/KodeMono-Bold.ttf")]
    pub font_kode_bold: Handle<Font>,
}
