use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct LondrinaFontAssets {
    pub w100_thin: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w900_black: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct SyneFontAssets {
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w800_extrabold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct OverlockFontAssets {
    pub w400_regular: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w900_black: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w700i_bold: Handle<Font>,
    pub w900i_black: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct ChakraPetchAssets {
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w300i_light: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w500i_medium: Handle<Font>,
    pub w600i_semibold: Handle<Font>,
    pub w700i_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct TitilliumWebAssets {
    pub w200_extralight: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w900_black: Handle<Font>,
    pub w200i_extralight: Handle<Font>,
    pub w300i_light: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w600i_semibold: Handle<Font>,
    pub w700i_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct VictorMonoAssets {
    pub w100_thin: Handle<Font>,
    pub w200_extralight: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
    pub w100i_thin: Handle<Font>,
    pub w200i_extralight: Handle<Font>,
    pub w300i_light: Handle<Font>,
    pub w400i_regular: Handle<Font>,
    pub w500i_medium: Handle<Font>,
    pub w600i_semibold: Handle<Font>,
    pub w700i_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct KodeMonoAssets {
    pub w400_regular: Handle<Font>,
    pub w500_medium: Handle<Font>,
    pub w600_semibold: Handle<Font>,
    pub w700_bold: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct FontAssets {
    pub londrina: LondrinaFontAssets,
    pub syne: SyneFontAssets,
    pub overlock: OverlockFontAssets,
    pub chakra: ChakraPetchAssets,
    pub titillium: TitilliumWebAssets,
    pub victormono: VictorMonoAssets,
    pub kodemono: KodeMonoAssets,
}
