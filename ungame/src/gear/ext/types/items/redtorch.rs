use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct RedTorch {
    pub enabled: bool,
}

impl GearUsable for RedTorch {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::RedTorchOn,
            false => GearSpriteID::RedTorchOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Red Torch"
    }

    fn get_description(&self) -> &'static str {
        "A simple red light used by astronomers to see on the dark without losing night vision eye's adaptation. But the ghost might also react to this too."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Battery: 40%".to_string()
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

    fn power(&self) -> f32 {
        match self.enabled {
            false => 0.0,
            true => 2.5,
        }
    }

    fn color(&self) -> Color {
        // Red
        Color::srgb(1.0, 0.20, 0.07)
    }
}

impl From<RedTorch> for Gear {
    fn from(value: RedTorch) -> Self {
        Gear::new_from_kind(GearKind::RedTorch(value.box_clone()))
    }
}
