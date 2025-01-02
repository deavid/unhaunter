use super::traits::GearUsable;
use uncore::types::evidence::{Evidence, EvidenceError};

/// Represents the different types of gear available in the game.
///
/// Each variant holds a specific gear struct with its own attributes and behavior.
#[derive(Debug, Default, Clone)]
pub enum GearKind {
    Thermometer(Box<dyn GearUsable>),
    EMFMeter(Box<dyn GearUsable>),
    Recorder(Box<dyn GearUsable>),
    Flashlight(Box<dyn GearUsable>),
    GeigerCounter(Box<dyn GearUsable>),
    UVTorch(Box<dyn GearUsable>),
    IonMeter(Box<dyn GearUsable>),
    SpiritBox(Box<dyn GearUsable>),
    ThermalImager(Box<dyn GearUsable>),
    RedTorch(Box<dyn GearUsable>),
    Photocam(Box<dyn GearUsable>),
    Compass(Box<dyn GearUsable>),
    EStaticMeter(Box<dyn GearUsable>),
    Videocam(Box<dyn GearUsable>),
    MotionSensor(Box<dyn GearUsable>),
    RepellentFlask(Box<dyn GearUsable>),
    QuartzStone(Box<dyn GearUsable>),
    Salt(Box<dyn GearUsable>),
    SageBundle(Box<dyn GearUsable>),
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
