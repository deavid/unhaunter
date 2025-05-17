use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct MotionSensor {
    pub enabled: bool,
}

impl GearUsable for MotionSensor {
    fn can_enable(&self) -> bool {
        // Motion sensor has no battery or glitch mechanics, can always be toggled if off.
        true
    }

    fn is_enabled(&self) -> bool {
        // Is truly enabled if the switch is on.
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.is_enabled() {
            // Use is_enabled for consistency
            true => GearSpriteID::MotionSensor, // Assuming MotionSensor sprite is for active state
            false => GearSpriteID::MotionSensor, // Or a specific off_sprite if available, current code shows same for both
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Motion Sensor"
    }

    fn get_description(&self) -> &'static str {
        "Shoots an infrared beam that if cut will make the device beep. Can alert if a presence passes through."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled); // Show ON/OFF based on the switch state
        let msg = if self.is_enabled() {
            // Use is_enabled for actual operational status
            "--".to_string() // Status when on
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        if self.is_enabled() {
            // If currently on
            self.enabled = false; // Turn it off
        } else if self.can_enable() {
            // If currently off and can be turned on
            self.enabled = true; // Turn it on
        }
        // This simplifies to self.enabled = !self.enabled for MotionSensor
        // as can_enable is always true.
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<MotionSensor> for Gear {
    fn from(value: MotionSensor) -> Self {
        Gear::new_from_kind(GearKind::MotionSensor, value.box_clone())
    }
}
