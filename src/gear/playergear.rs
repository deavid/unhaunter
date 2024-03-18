use crate::ghost_definitions::GhostType;

use super::{Gear, GearKind};
use bevy::prelude::*;
use enum_iterator::Sequence;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence)]
pub enum Hand {
    Left,
    Right,
}

#[derive(Component, Debug, Clone)]
pub struct InventoryNext {
    pub idx: usize,
}

impl InventoryNext {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
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
        use super::geigercounter::GeigerCounter;
        // use super::ionmeter::IonMeter;
        // use super::motionsensor::MotionSensor;
        // use super::photocam::Photocam;
        use super::recorder::Recorder;
        use super::redtorch::RedTorch;
        use super::spiritbox::SpiritBox;
        // use super::thermalimager::ThermalImager;
        use super::repellentflask::RepellentFlask;
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
                GeigerCounter::default().into(),
                RepellentFlask::default().into(),
                // Incomplete equipment:
                // IonMeter::default().into(),
                // ThermalImager::default().into(),
                // Photocam::default().into(),
                // Compass::default().into(),
                // EStaticMeter::default().into(),
                // MotionSensor::default().into(),
            ],
        }
    }

    /// Returns the nth next item in the inventory. If out of bounds, returns
    /// None. This is different from Some(Gear::None) which means that the slot
    /// exists but it is empty.
    pub fn get_next(&self, idx: usize) -> Option<Gear> {
        self.inventory.get(idx).cloned()
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
    pub fn craft_repellent(&mut self, ghost_type: GhostType) {
        use super::repellentflask::RepellentFlask;

        // Check if the repellent exists in inventory, if not, create it:
        let flask_exists = self
            .as_vec()
            .iter()
            .any(|x| matches!(x.0.kind, GearKind::RepellentFlask(_)));
        if !flask_exists {
            self.inventory.push(RepellentFlask::default().into());
        }

        // Assume that one always exists. Retrieve the &mut reference:

        let mut inventory = self.as_vec_mut();
        let Some(flask) = inventory
            .iter_mut()
            .find(|x| matches!(x.0.kind, GearKind::RepellentFlask(_)))
        else {
            error!("Flask not found??");
            return;
        };

        if let GearKind::RepellentFlask(ref mut flask) = flask.0.kind {
            flask.fill_liquid(ghost_type);
        }
    }

    pub fn can_craft_repellent(&self, ghost_type: GhostType) -> bool {
        // Check if the repellent exists in inventory, if not, create it:
        let flask_exists = self
            .as_vec()
            .iter()
            .any(|x| matches!(x.0.kind, GearKind::RepellentFlask(_)));
        if !flask_exists {
            return true;
        }

        // Assume that one always exists. Retrieve the &mut reference:
        let inventory = self.as_vec();
        let Some(flask) = inventory
            .iter()
            .find(|x| matches!(x.0.kind, GearKind::RepellentFlask(_)))
        else {
            error!("Flask not found??");
            return false;
        };

        if let GearKind::RepellentFlask(flask) = &flask.0.kind {
            flask.can_fill_liquid(ghost_type)
        } else {
            false
        }
    }
}
