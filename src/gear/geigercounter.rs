use bevy::prelude::*;

use super::{on_off, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct GeigerCounter {
    pub enabled: bool,
}

impl GearUsable for GeigerCounter {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::GeigerOn,
            false => GearSpriteID::GeigerOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Geiger Counter"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Clicks: 15/s".to_string()
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
