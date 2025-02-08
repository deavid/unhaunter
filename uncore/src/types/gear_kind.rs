use crate::types::evidence::{Evidence, EvidenceError};

/// Represents the different types of gear available in the game.
///
/// Each variant holds a specific gear struct with its own attributes and behavior.
#[derive(Debug, Default, Clone)]
pub enum GearKind {
    Thermometer,
    EMFMeter,
    Recorder,
    Flashlight,
    GeigerCounter,
    UVTorch,
    IonMeter,
    SpiritBox,
    ThermalImager,
    RedTorch,
    Photocam,
    Compass,
    EStaticMeter,
    Videocam,
    MotionSensor,
    RepellentFlask,
    QuartzStone,
    Salt,
    SageBundle,
    #[default]
    None,
}

impl GearKind {
    pub fn is_none(&self) -> bool {
        matches!(self, GearKind::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

impl TryFrom<&GearKind> for Evidence {
    type Error = EvidenceError;

    fn try_from(value: &GearKind) -> Result<Self, Self::Error> {
        match value {
            GearKind::Thermometer => Ok(Evidence::FreezingTemp),
            GearKind::EMFMeter => Ok(Evidence::EMFLevel5),
            GearKind::Recorder => Ok(Evidence::EVPRecording),
            GearKind::GeigerCounter => Ok(Evidence::CPM500),
            GearKind::UVTorch => Ok(Evidence::UVEctoplasm),
            GearKind::SpiritBox => Ok(Evidence::SpiritBox),
            GearKind::RedTorch => Ok(Evidence::RLPresence),
            GearKind::Videocam => Ok(Evidence::FloatingOrbs),
            GearKind::Flashlight => Err(EvidenceError::NoEvidenceForGear),
            GearKind::IonMeter => Err(EvidenceError::NoEvidenceForGear),
            GearKind::ThermalImager => Err(EvidenceError::NoEvidenceForGear),
            GearKind::Photocam => Err(EvidenceError::NoEvidenceForGear),
            GearKind::Compass => Err(EvidenceError::NoEvidenceForGear),
            GearKind::EStaticMeter => Err(EvidenceError::NoEvidenceForGear),
            GearKind::MotionSensor => Err(EvidenceError::NoEvidenceForGear),
            GearKind::RepellentFlask => Err(EvidenceError::NoEvidenceForGear),
            GearKind::QuartzStone => Err(EvidenceError::NoEvidenceForGear),
            GearKind::Salt => Err(EvidenceError::NoEvidenceForGear),
            GearKind::SageBundle => Err(EvidenceError::NoEvidenceForGear),
            GearKind::None => Err(EvidenceError::NoEvidenceForGear),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlayerGearKind {
    pub left_hand: GearKind,
    pub right_hand: GearKind,
    pub inventory: Vec<GearKind>,
}
