use crate::ghost::GhostSprite;

use super::super::systemparam::gearstuff::GearStuff;
use bevy::color::Color;
use uncore::{
    components::board::position::Position,
    types::{
        gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID},
        ghost::types::GhostType,
    },
};

/// Provides a common interface for all gear types, enabling consistent
/// interactions.
pub trait GearUsable: std::fmt::Debug + Sync + Send {
    /// Returns the display name of the gear (e.g., "EMF Reader").
    fn get_display_name(&self) -> &'static str;

    /// Returns a brief description of the gear's functionality.
    fn get_description(&self) -> &'static str;

    /// Returns a string representing the current status of the gear (e.g., "ON",
    /// "OFF", "Reading: 5.0 mG").
    fn get_status(&self) -> String;

    /// Triggers the gear's primary action (e.g., turn on/off, take a reading).
    fn set_trigger(&mut self, gs: &mut GearStuff);

    /// Updates the gear's internal state based on time, player actions, or game
    /// conditions.
    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}

    /// Returns the `GearSpriteID` for the gear's current state.
    fn get_sprite_idx(&self) -> GearSpriteID;

    /// Creates a boxed clone of the `GearUsable` object. (Unused for now)
    fn box_clone(&self) -> Box<dyn GearUsable>;

    /// Flashlight power
    fn power(&self) -> f32 {
        0.0
    }

    /// Flashlight color
    fn color(&self) -> Color {
        Color::srgb(0.0, 0.0, 0.0)
    }

    /// Repellent check
    fn can_fill_liquid(&self, _ghost_type: GhostType) -> bool {
        false
    }
    /// Repellent fill
    fn fill_liquid(&mut self, _ghost_type: GhostType) {}

    /// Quartz update helper
    fn aux_quartz_update(
        &mut self,
        _gear_pos: &Position,
        _ghost_pos: &Position,
        _ghost_sprite: &mut GhostSprite,
        _dt: f32,
    ) {
    }
}

impl Clone for Box<dyn GearUsable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
