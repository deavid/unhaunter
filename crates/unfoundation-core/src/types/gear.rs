use crate::types::evidence::{Evidence, EvidenceError};
use bevy::prelude::{Component, Reflect, ReflectComponent};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub enum EquipmentPosition {
    Hand(Hand),
    Stowed,
    // Van,
    Deployed,
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Sequence, Serialize, Deserialize, Component, Reflect,
)]
#[reflect(Component)]
pub enum Hand {
    Left,
    Right,
}

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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Reflect)]
pub struct VisualKey(String);

impl VisualKey {
    pub const NONE: &'static str = "none";

    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<T: Into<String>> From<T> for VisualKey {
    fn from(s: T) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for VisualKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
