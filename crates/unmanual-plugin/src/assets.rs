use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct ManualAssets {
    #[asset(path = "fonts/LondrinaSolid-Regular.ttf")]
    pub font_londrina_light: Handle<Font>,
    #[asset(path = "fonts/ChakraPetch-Regular.ttf")]
    pub font_chakra_regular: Handle<Font>,
    #[asset(path = "fonts/ChakraPetch-SemiBold.ttf")]
    pub font_chakra_semibold: Handle<Font>,
}
