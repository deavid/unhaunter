use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::playergear::HeldObject;
use crate::types::gear::equipment::Hand;
use crate::types::gear::kind::GearKind;

/// Message sent by the client to report hand/inventory entity assignments.
#[derive(Debug, Clone, Serialize, Deserialize, Message, Reflect, Default)]
#[reflect(Default)]
pub struct ExportPlayerGearMessage {
    pub left_hand: Option<Entity>,
    pub right_hand: Option<Entity>,
    pub inventory: Vec<Entity>,
    pub held_item: Option<HeldObject>,
}

impl bevy::ecs::entity::MapEntities for ExportPlayerGearMessage {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, mapper: &mut M) {
        if let Some(left_hand) = self.left_hand.as_mut() {
            *left_hand = mapper.get_mapped(*left_hand);
        }
        if let Some(right_hand) = self.right_hand.as_mut() {
            *right_hand = mapper.get_mapped(*right_hand);
        }
        for gear_entity in &mut self.inventory {
            *gear_entity = mapper.get_mapped(*gear_entity);
        }
        if let Some(held_item) = self.held_item.as_mut() {
            held_item.entity = mapper.get_mapped(held_item.entity);
        }
    }
}

/// One loadout action a join client can request from the server during the truck phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TruckLoadoutAction {
    /// Equip a gear item from the van inventory into the first free slot.
    AddGear(GearKind),
    /// Unequip the item currently held in the given hand.
    ClearHand(Hand),
    /// Unequip the backpack item at the given index.
    ClearInventorySlot(usize),
}

/// Sent by a join client to request a loadout change during the truck phase.
///
/// The server processes each action and applies the same spawn/despawn logic
/// as the host-local `button_clicked` handler, ensuring the server's copy of
/// the player's `PlayerGear` reflects the chosen loadout.
/// Transmitted on `Channel::Ordered` for reliability.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct TruckLoadoutMessage {
    pub action: TruckLoadoutAction,
}
