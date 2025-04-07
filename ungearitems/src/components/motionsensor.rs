use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct MotionSensor {
    pub enabled: bool,
}

impl GearUsable for MotionSensor {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::MotionSensor,
            false => GearSpriteID::MotionSensor,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Motion Sensor"
    }

    fn get_description(&self) -> &'static str {
        "Shoots an infrared beam that if cut will make the device beep. Can alert if a presence passes through."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "--".to_string()
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

impl From<MotionSensor> for Gear {
    fn from(value: MotionSensor) -> Self {
        Gear::new_from_kind(GearKind::MotionSensor, value.box_clone())
    }
}
