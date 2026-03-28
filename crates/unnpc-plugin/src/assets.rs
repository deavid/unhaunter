use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub(crate) struct NpcAssets {
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Light.ttf")]
    pub font_londrina_light: Handle<Font>,
    #[asset(path = "fonts/syne/static/Syne-Regular.ttf")]
    pub font_syne_regular: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Light.ttf")]
    pub font_chakra_light: Handle<Font>,
}
