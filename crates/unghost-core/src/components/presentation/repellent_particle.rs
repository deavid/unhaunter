use bevy::prelude::*;
use uninvestigation_core::ghost::GhostType;
use unspatial_core::direction::Direction;

/// Local-only particle component for repellent visual effects.
/// Never replicated. Owned and ticked entirely by unghost-presentation.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct RepellentParticle {
    pub class: GhostType,
    pub life: f32,
    pub dir: Direction,
    pub hit_correct: bool,
    pub hit_incorrect: bool,
}

impl RepellentParticle {
    const MAX_LIFE: f32 = 30.0;
    pub const DEFAULT_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 0.15);

    pub fn new(class: GhostType) -> Self {
        Self {
            class,
            life: Self::MAX_LIFE,
            dir: Direction::zero(),
            hit_correct: false,
            hit_incorrect: false,
        }
    }

    pub fn life_factor(&self) -> f32 {
        self.life / Self::MAX_LIFE
    }
}
