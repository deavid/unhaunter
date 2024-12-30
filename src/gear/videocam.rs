use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Videocam {
    pub enabled: bool,
}

impl Videocam {
    pub fn power(&self) -> f32 {
        match self.enabled {
            false => 0.0,
            true => 35.0,
        }
    }

    pub fn color(&self) -> Color {
        // Green-Cyan (for NightVision)
        Color::srgb(0.01, 1.00, 0.70)
    }
}

impl GearUsable for Videocam {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::Videocam,
            false => GearSpriteID::Videocam,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Video Camera NV"
    }

    fn get_description(&self) -> &'static str {
        "Mainly used for its infrared night vision, it can also transmit images to the van in real time."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "NIGHT VISION ON".to_string()
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

impl From<Videocam> for Gear {
    fn from(value: Videocam) -> Self {
        Gear::new_from_kind(GearKind::Videocam(value))
    }
}
