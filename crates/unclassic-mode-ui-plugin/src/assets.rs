use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub(crate) struct GameUiAssets {
    #[asset(path = "img/title.png")]
    pub title: Handle<Image>,
    #[asset(path = "img/vignette.png")]
    pub vignette: Handle<Image>,

    #[asset(path = "fonts/chakra_petch/ChakraPetch-Light.ttf")]
    pub font_chakra_light: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Regular.ttf")]
    pub font_chakra_regular: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Italic.ttf")]
    pub font_chakra_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-SemiBold.ttf")]
    pub font_victor_semibold: Handle<Font>,
    #[asset(path = "fonts/overlock/Overlock-Regular.ttf")]
    pub font_overlock_regular: Handle<Font>,
}
