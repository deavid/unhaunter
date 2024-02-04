use bevy::prelude::*;
use enum_iterator::Sequence;

use super::{emfmeter::EMFMeter, flashlight::Flashlight, thermometer::Thermometer, Gear};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence)]
pub enum Hand {
    Left,
    Right,
}

#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub hand: Hand,
}

impl Inventory {
    pub fn new_left() -> Self {
        Inventory { hand: Hand::Left }
    }
    pub fn new_right() -> Self {
        Inventory { hand: Hand::Right }
    }
}

#[derive(Component, Debug, Clone)]
pub struct InventoryStats;

#[derive(Clone, Debug, Resource, Default)]
pub struct PlayerGear {
    pub left_hand: Gear,
    pub right_hand: Gear,
    pub inventory: Vec<Gear>,
}

impl PlayerGear {
    pub fn new() -> Self {
        Self {
            left_hand: Gear::Flashlight(Flashlight::default()),
            right_hand: Gear::Thermometer(Thermometer::default()),
            inventory: vec![
                Gear::EMFMeter(EMFMeter::default()),
                Gear::GeigerCounter,
                Gear::UVTorch,
                Gear::Recorder,
                Gear::IonMeter,
                Gear::SpiritBox,
                Gear::ThermalImager,
                Gear::RedTorch,
                Gear::Photocam,
                Gear::Compass,
                Gear::EStaticMeter,
                Gear::Videocam,
                Gear::MotionSensor,
            ],
        }
    }
    pub fn cycle(&mut self) {
        let old_right = self.right_hand.clone();
        let last_idx = self.inventory.len() - 1;
        self.right_hand = self.inventory[0].clone();
        for i in 0..last_idx {
            self.inventory[i] = self.inventory[i + 1].clone()
        }
        self.inventory[last_idx] = old_right;
    }
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.right_hand, &mut self.left_hand);
    }
    pub fn get_hand(&self, hand: &Hand) -> Gear {
        match hand {
            Hand::Left => self.left_hand.clone(),
            Hand::Right => self.right_hand.clone(),
        }
    }
}
