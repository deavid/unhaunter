use bevy::prelude::*;

use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct EMFMeter {
    pub enabled: bool,
}

impl GearUsable for EMFMeter {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::EMFMeter0,
            false => GearSpriteID::EMFMeterOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "EMF Meter"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Reading: 10mG EMF1".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<EMFMeter> for Gear {
    fn from(value: EMFMeter) -> Self {
        Gear::new_from_kind(GearKind::EMFMeter(value))
    }
}
