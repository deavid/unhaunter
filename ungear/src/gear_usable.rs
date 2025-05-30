use crate::gear_stuff::GearStuff;
use bevy::color::Color;
use std::any::Any;
use uncore::components::board::position::Position;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::types::gear::{equipmentposition::EquipmentPosition, spriteid::GearSpriteID};
use uncore::types::ghost::types::GhostType; // Added

/// Provides a common interface for all gear types, enabling consistent
/// interactions.
pub trait GearUsable: std::fmt::Debug + Sync + Send + Any {
    // Added Any trait bound
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
    fn do_fill_liquid(&mut self, _ghost_type: GhostType) {}

    /// Quartz update helper
    fn aux_quartz_update(
        &mut self,
        _gear_pos: &Position,
        _ghost_pos: &Position,
        _ghost_sprite: &mut GhostSprite,
        _dt: f32,
    ) {
    }

    /// Electromagnetic interference
    fn apply_electromagnetic_interference(&mut self, _warning_level: f32, _distance: f32) {
        // Default implementation - override in electronic gear
        // Default: no effect
    }

    /// Helper to check if gear is electronic (susceptible to EMI)
    fn is_electronic(&self) -> bool {
        false
    }

    /// Returns true if this gear requires darkness for optimal use.
    fn needs_darkness(&self) -> bool {
        false
    }

    /// Returns true if the gear is enabled.
    fn is_enabled(&self) -> bool {
        false
    }

    /// Returns true if the gear can be enabled.
    fn can_enable(&self) -> bool {
        true
    }

    /// Returns 1.0 if the gear is showing strong signal of evidence in the status text.
    fn is_status_text_showing_evidence(&self) -> f32 {
        0.0
    }
    /// Returns 1.0 if the gear is showing strong signal of evidence in the icon.
    fn is_icon_showing_evidence(&self) -> f32 {
        0.0
    }
    /// Returns 1.0 if the gear is reporting strong signal of evidence via sound.
    fn is_sound_showing_evidence(&self) -> f32 {
        0.0
    }

    /// Returns true if the gear's blinking hint is currently active.
    fn is_blinking_hint_active(&self) -> bool {
        false
    }

    // ATTENTION: The "as_any" methods are **NOT NEEDED** for downcasting.
    // .. **DO NOT ENABLE THEM**.
    // fn as_any(&self) -> &dyn Any;
    // fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn GearUsable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
