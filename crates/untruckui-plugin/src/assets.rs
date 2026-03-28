use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub(crate) struct TruckUiAssets {
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Light.ttf")]
    pub font_londrina_light: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Light.ttf")]
    pub font_chakra_light: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Regular.ttf")]
    pub font_titillium_regular: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-SemiBold.ttf")]
    pub font_titillium_semibold: Handle<Font>,
}
