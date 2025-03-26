use bevy::prelude::*;

use crate::types::gear::equipmentposition::Hand;

#[derive(Debug, Clone, Resource, Default)]
pub struct LookingGear {
    pub toggled: bool,
    pub held: bool,
}

impl LookingGear {
    pub fn hand(&self) -> Hand {
        let left_hand = self.toggled ^ self.held;
        match left_hand {
            true => Hand::Left,
            false => Hand::Right,
        }
    }
    pub fn toggle(&mut self) {
        self.toggled = !self.toggled;
    }
}
