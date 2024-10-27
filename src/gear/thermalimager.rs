use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct ThermalImager {
    pub enabled: bool,
}

impl GearUsable for ThermalImager {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::ThermalImagerOn,
            false => GearSpriteID::ThermalImagerOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Thermal Imager"
    }

    fn get_description(&self) -> &'static str {
        "Heat vision to see easily what's hot and what's cold. Might improve visibility of the paranormal and haunted objects."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Battery: 33%".to_string()
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

impl From<ThermalImager> for Gear {
    fn from(value: ThermalImager) -> Self {
        Gear::new_from_kind(GearKind::ThermalImager(value))
    }
}
