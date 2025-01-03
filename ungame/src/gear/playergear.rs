pub use uncore::components::player_inventory::{Inventory, InventoryNext, InventoryStats};
pub use uncore::types::gear::equipmentposition::{EquipmentPosition, Hand};
use uncore::types::gear_kind::PlayerGearKind;

use super::ext::types::{gear::Gear, uncore_gearkind::GearKind};
use uncore::components::player::HeldObject;
use uncore::types::ghost::types::GhostType;

use bevy::prelude::*;

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

    // TODO: Remove this code as it is unused.
    // pub fn new() -> Self {
    //     use super::prelude::Flashlight;
    //     Self {
    //         left_hand: Flashlight::default().into(),
    //         right_hand: Gear::none(),
    //         inventory: vec![Gear::none(), Gear::none()],
    //         held_item: None,
    //     }
    // }

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

    pub fn cycle(&mut self) {
        let old_right = self.right_hand.clone();
        let last_idx = self.inventory.len() - 1;
        let Some((lidx, inv)) = self
            .inventory
            .iter()
            .enumerate()
            .find(|(_n, e)| !matches!(e.kind, GearKind::None))
        else {
            return;
        };
        self.right_hand = inv.clone();
        for i in lidx..last_idx {
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

    pub fn craft_repellent(&mut self, ghost_type: GhostType) {
        use super::prelude::RepellentFlask;

        // Check if the repellent exists in inventory, if not, create it:
        let flask_exists = self
            .as_vec()
            .iter()
            .any(|x| matches!(x.0.kind, GearKind::RepellentFlask));
        if !flask_exists {
            let old_rh = self.take_hand(&Hand::Right);
            self.right_hand = RepellentFlask::default().into();
            self.append(old_rh);
        }

        // Assume that one always exists. Retrieve the &mut reference:
        let mut inventory = self.as_vec_mut();
        let Some(flask) = inventory
            .iter_mut()
            .find(|x| matches!(x.0.kind, GearKind::RepellentFlask))
        else {
            error!("Flask not found??");
            return;
        };
        flask.0.data.as_mut().unwrap().do_fill_liquid(ghost_type);
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

impl From<PlayerGearKind> for PlayerGear {
    fn from(value: PlayerGearKind) -> Self {
        Self {
            left_hand: value.left_hand.into(),
            right_hand: value.right_hand.into(),
            inventory: value.inventory.into_iter().map(Gear::from).collect(),
            held_item: None,
        }
    }
}
