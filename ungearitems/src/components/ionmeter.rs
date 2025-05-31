use super::{Gear, GearKind, on_off};
use bevy::prelude::*;
use uncore::types::gear::spriteid::GearSpriteID;
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct IonMeter {
    pub enabled: bool,
}

impl GearUsable for IonMeter {
    fn can_enable(&self) -> bool {
        // IonMeter has no battery or glitch mechanics, so it can always be enabled if it's off.
        true
    }

    fn is_enabled(&self) -> bool {
        // Is truly enabled if the switch is on. No other conditions apply.
        self.enabled
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.is_enabled() {
            // Use is_enabled for consistency
            true => GearSpriteID::IonMeter0,
            false => GearSpriteID::IonMeterOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Ion Meter"
    }

    fn get_description(&self) -> &'static str {
        "Detects charged particles in the air. Ghost leave a trace as they move and this tool may help following the ghost."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled); // Show ON/OFF based on the switch state
        let msg = if self.is_enabled() {
            // Use is_enabled for actual operational status
            "Reading: 32eV".to_string() // Example status when on
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        if self.is_enabled() {
            // If currently on
            self.enabled = false; // Turn it off
        } else if self.can_enable() {
            // If currently off and can be turned on
            self.enabled = true; // Turn it on
        }
        // If off and cannot be enabled (though can_enable is always true here), it remains off.
        // This logic simplifies to self.enabled = !self.enabled for IonMeter.
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<IonMeter> for Gear {
    fn from(value: IonMeter) -> Self {
        Gear::new_from_kind(GearKind::IonMeter, value.box_clone())
    }
}
