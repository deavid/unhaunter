use bevy::prelude::*;

use crate::types::gear::equipment::Hand;
use crate::types::gear::kind::GearKind;

/// Emitted by `untruck-plugin` when the authority player requests to equip a gear item from the van.
/// Handled by `ungear-plugin` on the authority node.
#[derive(Clone, Debug, Message)]
pub struct RequestEquipGearFromVan {
    pub kind: GearKind,
}

/// Emitted by `untruck-plugin` when the authority player requests to unequip a hand slot.
/// Handled by `ungear-plugin` on the authority node.
#[derive(Clone, Debug, Message)]
pub struct RequestUnequipHand {
    pub hand: Hand,
}

/// Emitted by `untruck-plugin` when the authority player requests to unequip a backpack slot.
/// Handled by `ungear-plugin` on the authority node.
#[derive(Clone, Debug, Message)]
pub struct RequestUnequipInventorySlot {
    pub idx: usize,
}
