use super::board::direction::Direction;
use crate::types::ghost::types::GhostType;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct RepellentParticle {
    pub class: GhostType,
    pub life: f32,
    pub dir: Direction,
}

impl RepellentParticle {
    const MAX_LIFE: f32 = 30.0;

    pub fn new(class: GhostType) -> Self {
        Self {
            class,
            life: Self::MAX_LIFE,
            dir: Direction::zero(),
        }
    }

    pub fn life_factor(&self) -> f32 {
        self.life / Self::MAX_LIFE
    }
}
