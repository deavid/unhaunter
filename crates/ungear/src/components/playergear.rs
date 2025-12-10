use crate::types::gear::Gear;
use bevy::prelude::*;
use uncore::components::player::HeldObject;
use uncore::types::gear::equipmentposition::{EquipmentPosition, Hand};
use uncore::types::gear_kind::GearKind;
use uncore::types::ghost::types::GhostType;

#[derive(Clone, Debug, Component, Default)]
pub struct PlayerGear {
    pub left_hand: Gear,
    pub right_hand: Gear,
    pub inventory: Vec<Gear>,
    pub held_item: Option<HeldObject>,
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

    pub fn append(&mut self, ngear: Gear) {
        for (pgear, _hand) in self.as_vec_mut() {
            if matches!(pgear.kind, GearKind::None) {
                *pgear = ngear;
                return;
            }
        }
    }
    pub fn empty_right_handed(&self) -> bool {
        matches!(self.right_hand.kind, GearKind::None)
    }

    /// Returns the nth next item in the inventory. If out of bounds, returns None.
    /// This is different from Some(Gear::None) which means that the slot exists but it
    /// is empty.
    pub fn get_next(&self, idx: usize) -> Option<Gear> {
        self.inventory.get(idx).cloned()
    }

    pub fn get_next_non_empty(&self) -> Option<Gear> {
        self.inventory
            .iter()
            .find(|e| !matches!(e.kind, GearKind::None))
            .cloned()
    }

    pub fn take_next(&mut self, idx: usize) -> Option<Gear> {
        self.inventory.get_mut(idx).map(|g| {
            let mut ret = Gear::none();
            std::mem::swap(&mut ret, g);
            ret
        })
    }

    pub fn cycle(&mut self, hand: &Hand) {
        let old_hand = self.get_hand(hand).clone();
        let last_idx = self.inventory.len() - 1;
        let Some((lidx, inv)) = self
            .inventory
            .iter()
            .enumerate()
            .find(|(_n, e)| !matches!(e.kind, GearKind::None))
        else {
            return;
        };
        match hand {
            Hand::Left => self.left_hand = inv.clone(),
            Hand::Right => self.right_hand = inv.clone(),
        };
        for i in lidx..last_idx {
            self.inventory[i] = self.inventory[i + 1].clone()
        }
        self.inventory[last_idx] = old_hand;
    }

    /// Cycle gear in reverse direction (backwards through inventory)
    pub fn cycle_reverse(&mut self, hand: &Hand) {
        let old_hand = self.get_hand(hand).clone();

        // Find the last non-empty item in inventory (cycling backwards)
        let Some((lidx, inv)) = self
            .inventory
            .iter()
            .enumerate()
            .rev()
            .find(|(_n, e)| !matches!(e.kind, GearKind::None))
        else {
            return;
        };

        match hand {
            Hand::Left => self.left_hand = inv.clone(),
            Hand::Right => self.right_hand = inv.clone(),
        };

        // Shift items forward (opposite of cycle)
        for i in (1..=lidx).rev() {
            self.inventory[i] = self.inventory[i - 1].clone()
        }
        self.inventory[0] = old_hand;
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

    pub fn take_hand(&mut self, hand: &Hand) -> Gear {
        let mut ret = Gear::none();
        std::mem::swap(
            &mut ret,
            match hand {
                Hand::Left => &mut self.left_hand,
                Hand::Right => &mut self.right_hand,
            },
        );
        ret
    }

    pub fn can_craft_repellent(&self, ghost_type: GhostType) -> bool {
        // Check if the repellent exists in inventory, if not, create it:
        let flask_exists = self
            .as_vec()
            .iter()
            .any(|x| matches!(x.0.kind, GearKind::RepellentFlask));
        if !flask_exists {
            return true;
        }

        // Assume that one always exists. Retrieve the &mut reference:
        let inventory = self.as_vec();
        let Some(flask) = inventory
            .iter()
            .find(|x| matches!(x.0.kind, GearKind::RepellentFlask))
        else {
            error!("Flask not found??");
            return false;
        };
        flask.0.data.as_ref().unwrap().can_fill_liquid(ghost_type)
    }
}
