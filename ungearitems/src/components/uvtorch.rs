use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct UVTorch {
    pub enabled: bool,
}

impl GearUsable for UVTorch {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::UVTorchOn,
            false => GearSpriteID::UVTorchOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "UV Torch"
    }

    fn get_description(&self) -> &'static str {
        "Ultraviolet light that can be used to expose evidence invisible to the naked eye since some substances react to it and glow."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Battery: 47%".to_string()
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
            true => 2.0,
        }
    }

    fn color(&self) -> Color {
        // Violet
        Color::srgb(0.60, 0.25, 1.00)
    }
}

impl From<UVTorch> for Gear {
    fn from(value: UVTorch) -> Self {
        Gear::new_from_kind(GearKind::UVTorch, value.box_clone())
    }
}
