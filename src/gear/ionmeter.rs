use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct IonMeter {
    pub enabled: bool,
}

impl GearUsable for IonMeter {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
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
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Reading: 32eV".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn _box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<IonMeter> for Gear {
    fn from(value: IonMeter) -> Self {
        Gear::new_from_kind(GearKind::IonMeter(value))
    }
}
