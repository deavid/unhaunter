use bevy::utils::HashSet;
use enum_iterator::Sequence;
use enum_iterator::all;
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
            Evidence::FreezingTemp => {
                "The ghost and breach makes the ambient colder.\nSome ghosts will make the temperature drop below 0.0ºC."
            }
            Evidence::FloatingOrbs => {
                "Check if the breach lights up under Night vision.\nLights need to be off."
            }
            Evidence::UVEctoplasm => {
                "Check if the ghost turns green under UV.\nLights need to be off."
            }
            Evidence::EMFLevel5 => {
                "Some ghosts will register EMF5 on the meter.\nFollow the ghost close by and keep an eye on the reading."
            }
            Evidence::EVPRecording => {
                "Some ghost leave recordings. Keep an eye on the recorder.\nIf a EVP Recording is made, [EVP RECORDED] will appear."
            }
            Evidence::SpiritBox => {
                "Some ghosts talk trough the SpiritBox, specially near the breach and in darkness.\nIf you hear the ghost talking through it, mark this evidence."
            }
            Evidence::RLPresence => {
                "Some ghosts glow orange under red light.\nLights need to be off."
            }
            Evidence::CPM500 => {
                "Some ghosts are radioactive and will register above than 500cpm.\nIt takes time for the Geiger counter to settle into a value."
            }
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
