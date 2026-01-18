use bevy::prelude::*;
use unfoundation_core::types::gear::VisualKey;
use unghost_core::types::evidence::Evidence;

/// The display name of an item.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct ItemName(pub String);

impl ItemName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// A brief description of the item's functionality.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct ItemDescription(pub String);

impl ItemDescription {
    pub fn new(desc: impl Into<String>) -> Self {
        Self(desc.into())
    }
}

/// The sprite index for the gear.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct GearSprite(pub VisualKey);

/// Marker for items that are electronic and susceptible to EMI.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct Electronic {
    /// 0.0 = immune, 1.0 = highly sensitive
    pub sensitivity: f32,
    /// Time remaining for glitch effect
    pub glitch_timer: f32,
    /// Current intensity (0.0 - 1.0)
    pub glitch_intensity: f32,
}

/// Battery functionality for powered items.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct Battery {
    /// 0.0 to 1.0
    pub level: f32,
    /// Drain rate per second when active
    pub drain_rate: f32,
}

/// Evidence sensor functionality (EMF, Thermometer, etc.)
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct EvidenceSensor {
    pub evidence: Evidence,
}

/// Tracks what evidence a piece of gear is currently "showing" to the player.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct PerceivedClarity {
    /// 1.0 if status text is showing evidence, 0.0 otherwise.
    pub from_status_text: f32,
    /// 1.0 if the icon/sprite is showing evidence, 0.0 otherwise.
    pub from_icon: f32,
    /// 1.0 if audio is indicating evidence, 0.0 otherwise.
    pub from_sound: f32,
}

/// Marker for items that can be held in hands.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct Handheld;

/// Current status text of the gear (e.g. "Reading: 5.0 mG")
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct StatusText(pub String);
