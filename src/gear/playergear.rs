use bevy::prelude::*;
use enum_iterator::Sequence;

use super::Gear;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentPosition {
    Hand(Hand),
    Stowed,
    // Van,
    // Ground,
}

#[derive(Component, Debug, Clone)]
pub struct InventoryStats;

#[derive(Clone, Debug, Component, Default)]
pub struct PlayerGear {
    pub left_hand: Gear,
    pub right_hand: Gear,
    pub inventory: Vec<Gear>,
}

impl PlayerGear {
    pub fn as_vec(&self) -> Vec<(&Gear, EquipmentPosition)> {
        let mut ret = vec![
            (&self.left_hand, EquipmentPosition::Hand(Hand::Left)),
            (&self.right_hand, EquipmentPosition::Hand(Hand::Right)),
        ];
        for g in self.inventory.iter() {
            ret.push((g, EquipmentPosition::Stowed));
        }
        ret
    }
    pub fn as_vec_mut(&mut self) -> Vec<(&mut Gear, EquipmentPosition)> {
        let mut ret = vec![
            (&mut self.left_hand, EquipmentPosition::Hand(Hand::Left)),
            (&mut self.right_hand, EquipmentPosition::Hand(Hand::Right)),
        ];
        for g in self.inventory.iter_mut() {
            ret.push((g, EquipmentPosition::Stowed));
        }
        ret
    }
    pub fn new() -> Self {
        // use super::compass::Compass;
        use super::emfmeter::EMFMeter;
        // use super::estaticmeter::EStaticMeter;
        use super::flashlight::Flashlight;
        // use super::geigercounter::GeigerCounter;
        // use super::ionmeter::IonMeter;
        // use super::motionsensor::MotionSensor;
        // use super::photocam::Photocam;
        use super::recorder::Recorder;
        use super::redtorch::RedTorch;
        use super::spiritbox::SpiritBox;
        // use super::thermalimager::ThermalImager;
        use super::thermometer::Thermometer;
        use super::uvtorch::UVTorch;
        use super::videocam::Videocam;

        Self {
            left_hand: Flashlight::default().into(),
            right_hand: Thermometer::default().into(),
            inventory: vec![
                EMFMeter::default().into(),
                UVTorch::default().into(),
                SpiritBox::default().into(),
                Recorder::default().into(),
                Videocam::default().into(),
                RedTorch::default().into(),
                // Incomplete equipment:
                // GeigerCounter::default().into(),
                // IonMeter::default().into(),
                // ThermalImager::default().into(),
                // Photocam::default().into(),
                // Compass::default().into(),
                // EStaticMeter::default().into(),
                // MotionSensor::default().into(),
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
