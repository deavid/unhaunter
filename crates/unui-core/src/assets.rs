use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Debug, Clone)]
pub struct UiAssets {
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
    #[asset(path = "img/badges.png")]
    pub badges: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 5, rows = 1))]
    pub badges_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "img/vignette.png")]
    pub vignette: Handle<Image>,

    // Fonts
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Thin.ttf")]
    pub font_londrina_thin: Handle<Font>,
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Light.ttf")]
    pub font_londrina_light: Handle<Font>,
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Regular.ttf")]
    pub font_londrina_regular: Handle<Font>,
    #[asset(path = "fonts/londrina_solid/LondrinaSolid-Black.ttf")]
    pub font_londrina_black: Handle<Font>,

    #[asset(path = "fonts/syne/static/Syne-Regular.ttf")]
    pub font_syne_regular: Handle<Font>,
    #[asset(path = "fonts/syne/static/Syne-Medium.ttf")]
    pub font_syne_medium: Handle<Font>,
    #[asset(path = "fonts/syne/static/Syne-SemiBold.ttf")]
    pub font_syne_semibold: Handle<Font>,
    #[asset(path = "fonts/syne/static/Syne-Bold.ttf")]
    pub font_syne_bold: Handle<Font>,
    #[asset(path = "fonts/syne/static/Syne-ExtraBold.ttf")]
    pub font_syne_extrabold: Handle<Font>,

    #[asset(path = "fonts/overlock/Overlock-Regular.ttf")]
    pub font_overlock_regular: Handle<Font>,
    #[asset(path = "fonts/overlock/Overlock-Bold.ttf")]
    pub font_overlock_bold: Handle<Font>,
    #[asset(path = "fonts/overlock/Overlock-Black.ttf")]
    pub font_overlock_black: Handle<Font>,
    #[asset(path = "fonts/overlock/Overlock-Italic.ttf")]
    pub font_overlock_italic: Handle<Font>,
    #[asset(path = "fonts/overlock/Overlock-BoldItalic.ttf")]
    pub font_overlock_bold_italic: Handle<Font>,
    #[asset(path = "fonts/overlock/Overlock-BlackItalic.ttf")]
    pub font_overlock_black_italic: Handle<Font>,

    #[asset(path = "fonts/chakra_petch/ChakraPetch-Light.ttf")]
    pub font_chakra_light: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Regular.ttf")]
    pub font_chakra_regular: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Medium.ttf")]
    pub font_chakra_medium: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-SemiBold.ttf")]
    pub font_chakra_semibold: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Bold.ttf")]
    pub font_chakra_bold: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-LightItalic.ttf")]
    pub font_chakra_light_italic: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-Italic.ttf")]
    pub font_chakra_italic: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-MediumItalic.ttf")]
    pub font_chakra_medium_italic: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-SemiBoldItalic.ttf")]
    pub font_chakra_semibold_italic: Handle<Font>,
    #[asset(path = "fonts/chakra_petch/ChakraPetch-BoldItalic.ttf")]
    pub font_chakra_bold_italic: Handle<Font>,

    #[asset(path = "fonts/titillium_web/TitilliumWeb-ExtraLight.ttf")]
    pub font_titillium_extralight: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Light.ttf")]
    pub font_titillium_light: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Regular.ttf")]
    pub font_titillium_regular: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-SemiBold.ttf")]
    pub font_titillium_semibold: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Bold.ttf")]
    pub font_titillium_bold: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Black.ttf")]
    pub font_titillium_black: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-ExtraLightItalic.ttf")]
    pub font_titillium_extralight_italic: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-LightItalic.ttf")]
    pub font_titillium_light_italic: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-Italic.ttf")]
    pub font_titillium_italic: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-SemiBoldItalic.ttf")]
    pub font_titillium_semibold_italic: Handle<Font>,
    #[asset(path = "fonts/titillium_web/TitilliumWeb-BoldItalic.ttf")]
    pub font_titillium_bold_italic: Handle<Font>,

    #[asset(path = "fonts/victor_mono/static/VictorMono-Thin.ttf")]
    pub font_victor_thin: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-ExtraLight.ttf")]
    pub font_victor_extralight: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-Light.ttf")]
    pub font_victor_light: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-Regular.ttf")]
    pub font_victor_regular: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-Medium.ttf")]
    pub font_victor_medium: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-SemiBold.ttf")]
    pub font_victor_semibold: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-Bold.ttf")]
    pub font_victor_bold: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-ThinItalic.ttf")]
    pub font_victor_thin_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-ExtraLightItalic.ttf")]
    pub font_victor_extralight_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-LightItalic.ttf")]
    pub font_victor_light_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-Italic.ttf")]
    pub font_victor_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-MediumItalic.ttf")]
    pub font_victor_medium_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-SemiBoldItalic.ttf")]
    pub font_victor_semibold_italic: Handle<Font>,
    #[asset(path = "fonts/victor_mono/static/VictorMono-BoldItalic.ttf")]
    pub font_victor_bold_italic: Handle<Font>,

    #[asset(path = "fonts/kode_mono/static/KodeMono-Regular.ttf")]
    pub font_kode_regular: Handle<Font>,
    #[asset(path = "fonts/kode_mono/static/KodeMono-Medium.ttf")]
    pub font_kode_medium: Handle<Font>,
    #[asset(path = "fonts/kode_mono/static/KodeMono-SemiBold.ttf")]
    pub font_kode_semibold: Handle<Font>,
    #[asset(path = "fonts/kode_mono/static/KodeMono-Bold.ttf")]
    pub font_kode_bold: Handle<Font>,
}
