use crate::gear::GearKind;
use bevy::utils::HashSet;
use enum_iterator::all;
use enum_iterator::Sequence;
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Sequence)]
pub enum Evidence {
    FreezingTemp,
    FloatingOrbs,
    UVEctoplasm,
    EMFLevel5,
    EVPRecording,
    SpiritBox,
    RLPresence,
    CPM500,
}

impl Evidence {
    pub fn all() -> enum_iterator::All<Evidence> {
        all::<Evidence>()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Evidence::FreezingTemp => "Freezing Temps",
            Evidence::FloatingOrbs => "Floating Orbs",
            Evidence::UVEctoplasm => "UV Ectoplasm",
            Evidence::EMFLevel5 => "EMF Level 5",
            Evidence::EVPRecording => "EVP Recording",
            Evidence::SpiritBox => "Spirit Box",
            Evidence::RLPresence => "RL Presence",
            Evidence::CPM500 => "500+ cpm",
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            Evidence
            ::FreezingTemp => "The ghost and breach makes the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC.",
            Evidence::FloatingOrbs => "Check if the breach lights up under Night vision.\nLights need to be off.",
            Evidence::UVEctoplasm => "Check if the ghost turns green under UV.\nLights need to be off.",
            Evidence
            ::EMFLevel5 => "Some ghosts will register EMF5 on the meter.\nFollow the ghost close by and keep an eye on the reading.",
            Evidence
            ::EVPRecording => "Some ghost leave recordings. Keep an eye on the recorder.\nIf a EVP Recording is made, [EVP RECORDED] will appear.",
            Evidence
            ::SpiritBox => "Some ghosts talk trough the SpiritBox.\nIf you hear the ghost talking through it, mark this evidence.",
            Evidence::RLPresence => "Some ghosts glow orange under red light.\nLights need to be off.",
            Evidence
            ::CPM500 => "Some ghosts are radioactive and will register above than 500cpm.\nIt takes time for the Geiger counter to settle into a value.",
        }
    }

    pub fn from_bits(bits: u8) -> HashSet<Evidence> {
        let mut evidences = HashSet::new();
        let mut mask = 1;
        for evidence in all::<Evidence>() {
            if bits & mask != 0 {
                evidences.insert(evidence);
            }
            mask <<= 1;
        }
        evidences
    }
}

impl Display for Evidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Error)]
pub enum EvidenceError {
    #[error("No Evidence for Gear")]
    NoEvidenceForGear,
}

impl TryFrom<&GearKind> for Evidence {
    type Error = EvidenceError;

    fn try_from(value: &GearKind) -> Result<Self, Self::Error> {
        match value {
            GearKind::Thermometer(_) => Ok(Evidence::FreezingTemp),
            GearKind::EMFMeter(_) => Ok(Evidence::EMFLevel5),
            GearKind::Recorder(_) => Ok(Evidence::EVPRecording),
            GearKind::GeigerCounter(_) => Ok(Evidence::CPM500),
            GearKind::UVTorch(_) => Ok(Evidence::UVEctoplasm),
            GearKind::SpiritBox(_) => Ok(Evidence::SpiritBox),
            GearKind::RedTorch(_) => Ok(Evidence::RLPresence),
            GearKind::Videocam(_) => Ok(Evidence::FloatingOrbs),
            GearKind::Flashlight(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::IonMeter(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::ThermalImager(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::Photocam(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::Compass(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::EStaticMeter(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::MotionSensor(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::RepellentFlask(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::QuartzStone(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::Salt(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::SageBundle(_) => Err(EvidenceError::NoEvidenceForGear),
            GearKind::None => Err(EvidenceError::NoEvidenceForGear),
        }
    }
}

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
    Domovoi,
    Ghostlight,
    Kappa,
    Tengu,
    LaLlorona,
    Curupira,
    Dybbuk,
    Phooka,
    Wisp,
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
            GhostType::Domovoi => "Domovoi",
            GhostType::Ghostlight => "Ghostlight",
            GhostType::Kappa => "Kappa",
            GhostType::Tengu => "Tengu",
            GhostType::LaLlorona => "La Llorona",
            GhostType::Curupira => "Curupira",
            GhostType::Dybbuk => "Dybbuk",
            GhostType::Phooka => "Phooka",
            GhostType::Wisp => "Wisp",
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
            Domovoi => Evidence::from_bits(0b01011101),
            Ghostlight => Evidence::from_bits(0b01011110),
            Kappa => Evidence::from_bits(0b11100101),
            Tengu => Evidence::from_bits(0b01101011),
            LaLlorona => Evidence::from_bits(0b10111100),
            Curupira => Evidence::from_bits(0b01101110),
            Dybbuk => Evidence::from_bits(0b01110011),
            Phooka => Evidence::from_bits(0b01110101),
            Wisp => Evidence::from_bits(0b01110110),
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

impl Display for GhostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GhostSet {
    TmpEMF,
    TmpEMFUVOrbs,
    TmpEMFUVOrbsEVPCPM,
    Twenty,
    #[default]
    All,
}

impl GhostSet {
    pub fn as_vec(&self) -> Vec<GhostType> {
        use GhostType::*;

        match self {
            Self::TmpEMF => vec![LadyInWhite, BrownLady],
            Self::TmpEMFUVOrbs => vec![Caoilte, Ceara, Orla, Finvarra, Kappa],
            Self::TmpEMFUVOrbsEVPCPM => vec![
                Bugbear, Morag, Barghest, Boggart, Obayifo, WillOWisp, LaLlorona, Widow,
                Leprechaun, Brume,
            ],
            Self::Twenty => vec![
                Curupira, LaLlorona, Phooka, Obayifo, Maresca, Dybbuk, Caoilte, Orla, Jorogumo,
                Mider, Wisp, Cairbre, Ceara, Widow, BeanSidhe, Bugbear, Dullahan, Domovoi,
                Muirgheas, Namahage,
            ],
            Self::All => GhostType::all().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::utils::HashMap;

    #[test]
    fn test_generate_evidence_combinations() {
        for i in 0..256 {
            let mut nbits = 0;
            for n in 0..8 {
                if (i >> n) & 0x1 > 0 {
                    nbits += 1;
                }
            }
            if nbits == 5 {
                println!("0b{:08b}", i);
            }
        }
    }

    #[test]
    fn test_unique_evidence_combinations() {
        let mut all_combinations: HashSet<String> = HashSet::new();
        for ghost in all::<GhostType>() {
            let mut evidences = ghost
                .evidences()
                .into_iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            evidences.sort();
            let evidences = evidences.join("|");
            assert!(
                all_combinations.insert(evidences),
                "Found duplicate evidence set for {:?}",
                ghost
            );
        }
    }

    #[test]
    fn test_evidence_per_ghost() {
        for ghost in all::<GhostType>() {
            let evidences = ghost
                .evidences()
                .into_iter()
                .map(|x| x.name())
                .collect::<Vec<_>>();
            assert!(
                evidences.len() == 5,
                "The ghost {:?} does not have 5 evidences, instead it has: {:?}",
                ghost,
                evidences
            );
        }
    }

    #[test]
    fn test_balanced_evidence_usage() {
        let mut evidence_count: HashMap<Evidence, usize> = HashMap::new();
        for ghost in all::<GhostType>() {
            for &evidence in &ghost.evidences() {
                *evidence_count.entry(evidence).or_insert(0) += 1;
            }
        }

        // Assuming a balanced distribution, each evidence should be used roughly the same
        // number of times.
        let avg_use = evidence_count.values().sum::<usize>() / evidence_count.len();
        for (&evidence, &count) in &evidence_count {
            println!("Evidence {:?} used {} times", evidence, count);
        }
        for (&evidence, &count) in &evidence_count {
            assert!(
                (count as i32 - avg_use as i32).abs() <= 3,
                "Evidence {:?} is used an unbalanced number of times: {} (avg: {})",
                evidence,
                count,
                avg_use
            );
        }
    }
}
