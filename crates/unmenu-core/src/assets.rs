use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct MenuAssets {
    #[asset(path = "img/title.png")]
    pub title: Handle<Image>,
    #[asset(path = "img/menu-background.jpg")]
    pub menu_background: Handle<Image>,
    #[asset(path = "img/menu-background-low-contrast.jpg")]
    pub menu_background_low_contrast: Handle<Image>,
    #[asset(path = "img/scroll_arrow_up.png")]
    pub scroll_arrow_up: Handle<Image>,
    #[asset(path = "img/scroll_arrow_down.png")]
    pub scroll_arrow_down: Handle<Image>,
    #[asset(path = "img/scroll_thumb.png")]
    pub scroll_thumb: Handle<Image>,
    #[asset(path = "img/scroll_track.png")]
    pub scroll_track: Handle<Image>,

    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Light.ttf")]
    pub font_londrina_light: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Light.ttf")]
    pub font_titillium_light: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Regular.ttf")]
    pub font_titillium_regular: Handle<Font>,
    #[asset(path = "fonts/kode_mono/static/KodeMono-Bold.ttf")]
    pub font_kode_bold: Handle<Font>,
}
