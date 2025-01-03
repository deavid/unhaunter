use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Photocam {
    pub enabled: bool,
}

impl GearUsable for Photocam {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::PhotocamFlash2,
            false => GearSpriteID::Photocam,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Photo Camera"
    }

    fn get_description(&self) -> &'static str {
        "Takes photos, hopefully of something paranormal."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Flashy!".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Photocam> for Gear {
    fn from(value: Photocam) -> Self {
        Gear::new_from_kind(GearKind::Photocam,value.box_clone())
    }
}
