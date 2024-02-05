use bevy::prelude::*;
use rand::Rng;

use crate::board::Position;

use super::{on_off, playergear::EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone, Default)]
pub struct Thermometer {
    pub enabled: bool,
    pub temp: f32,
    pub temp_l2: [f32; 6],
    pub temp_l1: f32,
    pub frame_counter: u16,
}

impl GearUsable for Thermometer {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::ThermometerOn,
            false => GearSpriteID::ThermometerOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Thermometer"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            format!("Temperature: {:.1}ºC", self.temp)
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }
    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        // TODO: Add two thresholds: LO: -0.1 and HI: 5.1, with sound effects to notify + distintive icons.
        let mut rng = rand::thread_rng();
        self.frame_counter += 1;
        self.frame_counter %= 65413;
        const K: f32 = 0.5;
        let pos = Position {
            x: pos.x + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            y: pos.y + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            z: pos.z + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            global_z: pos.global_z,
        };
        let bpos = pos.to_board_position();
        let Some(light) = gs.bf.light_field.get(&bpos) else {
            return;
        };
        let temp_reading = light.lux * 7.0 - 1.0;
        const AIR_MASS: f32 = 20.0;
        // Double noise reduction to remove any noise from measurement.
        let n = self.frame_counter as usize % self.temp_l2.len();
        self.temp_l2[n] = (self.temp_l2[n] * AIR_MASS + self.temp_l1) / (AIR_MASS + 1.0);
        self.temp_l1 = (self.temp_l1 * AIR_MASS + temp_reading) / (AIR_MASS + 1.0);
        if self.frame_counter % 10 == 0 {
            let sum_temp: f32 = self.temp_l2.iter().sum();
            let avg_temp: f32 = sum_temp / self.temp_l2.len() as f32;
            self.temp = (avg_temp * 5.0).round() / 5.0;
        }
    }
    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Thermometer> for Gear {
    fn from(value: Thermometer) -> Self {
        Gear::new_from_kind(GearKind::Thermometer(value))
    }
}
