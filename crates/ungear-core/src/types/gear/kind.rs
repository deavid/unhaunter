use bevy::prelude::{Component, Reflect, ReflectComponent};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use unghost_core::types::evidence::Evidence;

/// Represents the different types of gear available in the game.
///
/// Each variant holds a specific gear struct with its own attributes and behavior.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Sequence,
    Component,
    Reflect,
)]
#[reflect(Component)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerGearKind {
    pub left_hand: GearKind,
    pub right_hand: GearKind,
    pub inventory: Vec<GearKind>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EvidenceError {
    #[error("No Evidence for Gear")]
    NoEvidenceForGear,
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
