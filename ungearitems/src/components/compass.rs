use super::{Gear, GearKind, GearSpriteID, on_off};
use bevy::prelude::*;
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Compass {
    pub enabled: bool,
}

impl GearUsable for Compass {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::Compass,
            false => GearSpriteID::Compass,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Compass"
    }

    fn get_description(&self) -> &'static str {
        "Measures the Earth's magnetic field, and sometimes the ghost."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "You managed to 'turn on' a compass".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Compass> for Gear {
    fn from(value: Compass) -> Self {
        Gear::new_from_kind(GearKind::Compass, value.box_clone())
    }
}
