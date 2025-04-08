use bevy::utils::HashSet;
use enum_iterator::{Sequence, all};

use crate::types::evidence::Evidence;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Sequence)]
pub enum GhostType {
    BeanSidhe,
    Dullahan,
    Leprechaun,
    Barghest,
    WillOWisp,
    Widow,
    HobsTally,
    Ghoul,
    Afrit,
    BaobhanSith,
    Ghostlight,
    Kappa,
    Tengu,
    LaLlorona,
    Curupira,
    Dybbuk,
    Phooka,
    Aswang,
    GrayMan,
    LadyInWhite,
    Maresca,
    Gashadokuro,
    Jorogumo,
    Namahage,
    Tsuchinoko,
    Obayifo,
    Brume,
    Bugbear,
    Boggart,
    GreyLady,
    OldNan,
    BrownLady,
    Morag,
    Fionnuala,
    Ailill,
    Cairbre,
    Oonagh,
    Mider,
    Orla,
    Finvarra,
    Caoilte,
    Ceara,
    Muirgheas,
    Domovoy,
}

impl GhostType {
    pub fn all() -> enum_iterator::All<GhostType> {
        all::<GhostType>()
    }

    pub fn name(&self) -> &'static str {
        match self {
            GhostType::BeanSidhe => "Bean Sidhe",
            GhostType::Dullahan => "Dullahan",
            GhostType::Leprechaun => "Leprechaun",
            GhostType::Barghest => "Barghest",
            GhostType::WillOWisp => "Will O'Wisp",
            GhostType::Widow => "Widow",
            GhostType::HobsTally => "Hobs Tally",
            GhostType::Ghoul => "Ghoul",
            GhostType::Afrit => "Afrit",
            GhostType::BaobhanSith => "BaobhanSith",
            GhostType::Ghostlight => "Ghostlight",
            GhostType::Kappa => "Kappa",
            GhostType::Tengu => "Tengu",
            GhostType::LaLlorona => "La Llorona",
            GhostType::Curupira => "Curupira",
            GhostType::Dybbuk => "Dybbuk",
            GhostType::Phooka => "Phooka",
            GhostType::Aswang => "Aswang",
            GhostType::GrayMan => "Gray Man",
            GhostType::LadyInWhite => "Lady in White",
            GhostType::Maresca => "Maresca",
            GhostType::Gashadokuro => "Gashadokuro",
            GhostType::Jorogumo => "Jorōgumo",
            GhostType::Namahage => "Namahage",
            GhostType::Tsuchinoko => "Tsuchinoko",
            GhostType::Obayifo => "Obayifo",
            GhostType::Brume => "Brume",
            GhostType::Bugbear => "Bugbear",
            GhostType::Boggart => "Boggart",
            GhostType::GreyLady => "Grey Lady",
            GhostType::OldNan => "Old Nan",
            GhostType::BrownLady => "Brown Lady",
            GhostType::Morag => "Morag",
            GhostType::Fionnuala => "Fionnuala",
            GhostType::Ailill => "Ailill",
            GhostType::Cairbre => "Cairbre",
            GhostType::Oonagh => "Oonagh",
            GhostType::Mider => "Mider",
            GhostType::Orla => "Orla",
            GhostType::Finvarra => "Finvarra",
            GhostType::Caoilte => "Caoilte",
            GhostType::Ceara => "Ceara",
            GhostType::Muirgheas => "Muirgheas",
            GhostType::Domovoy => "Domovoy",
        }
    }

    #[rustfmt::skip]
    pub fn evidences(&self) -> HashSet<Evidence> {
        use GhostType::*;

        // Order of evidence: (From right to left)
        //
        // * FreezingTemp, 1   0000 0001
        //
        // * FloatingOrbs, 2   0000 0010
        //
        // * UVEctoplasm,  3   0000 0100
        //
        // * EMFLevel5,    4   0000 1000
        //
        // * EVPRecording, 5   0001 0000
        //
        // * SpiritBox,    6   0010 0000
        //
        // * RLPresence,   7   0100 0000
        //
        // * CPM500,       8   1000 0000
        match self {
            // -------------------------------   87654321
            BeanSidhe => Evidence::from_bits(0b00011111),
            Dullahan => Evidence::from_bits(0b01101101),
            Leprechaun => Evidence::from_bits(0b00110111),
            Barghest => Evidence::from_bits(0b00111011),
            WillOWisp => Evidence::from_bits(0b00111101),
            Widow => Evidence::from_bits(0b00111110),
            HobsTally => Evidence::from_bits(0b01001111),
            Ghoul => Evidence::from_bits(0b01010111),
            Afrit => Evidence::from_bits(0b01011011),
            BaobhanSith => Evidence::from_bits(0b01011101),
            Ghostlight => Evidence::from_bits(0b01011110),
            Kappa => Evidence::from_bits(0b11100101),
            Tengu => Evidence::from_bits(0b01101011),
            LaLlorona => Evidence::from_bits(0b10111100),
            Curupira => Evidence::from_bits(0b01101110),
            Dybbuk => Evidence::from_bits(0b01110011),
            Phooka => Evidence::from_bits(0b01110101),
            Aswang => Evidence::from_bits(0b01110110),
            GrayMan => Evidence::from_bits(0b01111001),
            LadyInWhite => Evidence::from_bits(0b11110001),
            Maresca => Evidence::from_bits(0b10001111),
            Gashadokuro => Evidence::from_bits(0b10010111),
            Jorogumo => Evidence::from_bits(0b10011011),
            Namahage => Evidence::from_bits(0b10011101),
            Tsuchinoko => Evidence::from_bits(0b10011110),
            Obayifo => Evidence::from_bits(0b10100111),
            Brume => Evidence::from_bits(0b10101110),
            Bugbear => Evidence::from_bits(0b10101101),
            Boggart => Evidence::from_bits(0b10110011),
            GreyLady => Evidence::from_bits(0b10110101),
            OldNan => Evidence::from_bits(0b10110110),
            BrownLady => Evidence::from_bits(0b11111000),
            Morag => Evidence::from_bits(0b10111010),
            Fionnuala => Evidence::from_bits(0b11000111),
            Ailill => Evidence::from_bits(0b11001101),
            Cairbre => Evidence::from_bits(0b11010011),
            Oonagh => Evidence::from_bits(0b11010110),
            Mider => Evidence::from_bits(0b11011010),
            Orla => Evidence::from_bits(0b11100011),
            Finvarra => Evidence::from_bits(0b11100110),
            Caoilte => Evidence::from_bits(0b11101010),
            Ceara => Evidence::from_bits(0b11101100),
            Muirgheas => Evidence::from_bits(0b11110010),
            Domovoy => Evidence::from_bits(0b11110100),
        }
    }
}

impl std::fmt::Display for GhostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
