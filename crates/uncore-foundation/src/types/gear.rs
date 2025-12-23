use crate::types::evidence::{Evidence, EvidenceError};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentPosition {
    Hand(Hand),
    Stowed,
    // Van,
    Deployed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence, Serialize, Deserialize)]
pub enum Hand {
    Left,
    Right,
}

/// Represents the different types of gear available in the game.
///
/// Each variant holds a specific gear struct with its own attributes and behavior.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Sequence)]
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

    pub fn is_evidence_tool_for(&self, evidence_type: Evidence) -> bool {
        match self {
            GearKind::Thermometer => evidence_type == Evidence::FreezingTemp,
            GearKind::EMFMeter => evidence_type == Evidence::EMFLevel5,
            GearKind::Recorder => evidence_type == Evidence::EVPRecording,
            GearKind::GeigerCounter => evidence_type == Evidence::CPM500,
            GearKind::UVTorch => evidence_type == Evidence::UVEctoplasm,
            GearKind::SpiritBox => evidence_type == Evidence::SpiritBox,
            GearKind::RedTorch => evidence_type == Evidence::RLPresence,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerGearKind {
    pub left_hand: GearKind,
    pub right_hand: GearKind,
    pub inventory: Vec<GearKind>,
}

/// Unique identifiers for different gear sprites.
///
/// Each variant represents a specific sprite or animation frame for a piece of
/// gear. The values are used to index into the gear spritesheet.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum GearSpriteID {
    ThermometerOff = 0,
    ThermometerOn,
    ThermalImagerOff,
    ThermalImagerOn,
    EMFMeterOff = 10,
    EMFMeter0,
    EMFMeter1,
    EMFMeter2,
    EMFMeter3,
    EMFMeter4,
    RecorderOff = 20,
    Recorder1,
    Recorder2,
    Recorder3,
    Recorder4,
    FlashlightOff = 30,
    Flashlight1,
    Flashlight2,
    Flashlight3,
    GeigerOff,
    GeigerOn,
    GeigerTick,
    RedTorchOff = 40,
    RedTorchOn,
    UVTorchOff,
    UVTorchOn,
    Photocam,
    PhotocamFlash1,
    PhotocamFlash2,
    IonMeterOff = 50,
    IonMeter0,
    IonMeter1,
    IonMeter2,
    SpiritBoxOff,
    SpiritBoxScan1,
    SpiritBoxScan2,
    SpiritBoxScan3,
    SpiritBoxAns1,
    SpiritBoxAns2,
    RepelentFlaskEmpty = 60,
    RepelentFlaskFull,
    // Quartz Stone
    QuartzStone0 = 65,
    QuartzStone1,
    QuartzStone2,
    QuartzStone3,
    QuartzStone4,
    // Salt
    Salt4 = 75,
    Salt3,
    Salt2,
    Salt1,
    Salt0,
    Compass = 80,
    // Sage Bundle
    SageBundle0 = 85,
    SageBundle1,
    SageBundle2,
    SageBundle3,
    SageBundle4,
    EStaticMeter = 90,
    Videocam,
    MotionSensor,
    #[default]
    None,
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
            _ => Err(EvidenceError::NoEvidenceForGear),
        }
    }
}

impl TryFrom<GearKind> for Evidence {
    type Error = EvidenceError;

    fn try_from(value: GearKind) -> Result<Self, Self::Error> {
        Evidence::try_from(&value)
    }
}
