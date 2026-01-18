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
