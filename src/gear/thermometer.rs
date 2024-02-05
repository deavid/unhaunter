use bevy::prelude::*;
use rand::Rng;

use crate::board::Position;

use super::{on_off, playergear::EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone)]
pub struct Thermometer {
    pub enabled: bool,
    pub temp: f32,
    pub temp_l2: [f32; 20],
    pub temp_l1: f32,
    pub frame_counter: u16,
}

impl Default for Thermometer {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            temp: 10.0,
            temp_l2: [10.0; 20],
            temp_l1: 10.0,
            frame_counter: Default::default(),
        }
    }
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
        let Some(temperature) = gs.bf.temperature_field.get(&bpos) else {
            return;
        };
        let temp_reading = temperature;
        const AIR_MASS: f32 = 5.0;
        // Double noise reduction to remove any noise from measurement.
        let n = self.frame_counter as usize % self.temp_l2.len();
        self.temp_l2[n] = (self.temp_l2[n] * AIR_MASS + self.temp_l1) / (AIR_MASS + 1.0);
        self.temp_l1 = (self.temp_l1 * AIR_MASS + temp_reading) / (AIR_MASS + 1.0);
        if self.frame_counter % 20 == 0 {
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

pub fn temperature_update(
    mut bf: ResMut<crate::board::BoardData>,
    roomdb: Res<crate::board::RoomDB>,
    qt: Query<(&Position, &crate::behavior::Behavior)>,
) {
    let ambient = bf.ambient_temp;
    for (pos, bh) in qt.iter() {
        let bpos = pos.to_board_position();
        let t_out = bh.temp_heat_output();
        bf.temperature_field.entry(bpos).and_modify(|t| *t += t_out);
    }
    let old_temps: Vec<(_, _)> = bf
        .temperature_field
        .iter()
        .map(|(p, t)| (p.clone(), *t))
        .collect();
    let mut rng = rand::thread_rng();
    const OUTSIDE_CONDUCTIVITY: f32 = 100.0;
    const INSIDE_CONDUCTIVITY: f32 = 10.0;
    const WALL_CONDUCTIVITY: f32 = 0.02;
    const SMOOTH: f32 = 1.0;

    for (p, temp) in old_temps.into_iter() {
        let free = bf
            .collision_field
            .get(&p)
            .map(|x| x.player_free)
            .unwrap_or(true);
        let mut self_k = match free {
            true => INSIDE_CONDUCTIVITY,
            false => WALL_CONDUCTIVITY,
        };
        let is_outside = roomdb.room_tiles.get(&p).is_none();
        if is_outside {
            self_k = OUTSIDE_CONDUCTIVITY;
        }

        let neighbors = p.xy_neighbors(1);
        let n_idx = rng.gen_range(0..neighbors.len());
        let neigh = neighbors[n_idx].clone();

        let neigh_free = bf
            .collision_field
            .get(&neigh)
            .map(|x| x.player_free)
            .unwrap_or(true);
        let neigh_k = match neigh_free {
            true => INSIDE_CONDUCTIVITY,
            false => WALL_CONDUCTIVITY,
        };
        let nis_outside = roomdb.room_tiles.get(&neigh).is_none();
        if nis_outside {
            self_k = OUTSIDE_CONDUCTIVITY;
        }

        let neigh_temp = bf.temperature_field.get(&neigh).copied().unwrap_or(ambient);

        let mid_temp = (temp * self_k + neigh_temp * neigh_k) / (self_k + neigh_k);

        let conductivity = (self_k.recip() + neigh_k.recip()).recip() / SMOOTH;

        let new_temp1 = (temp + mid_temp * conductivity) / (conductivity + 1.0);
        let new_temp2 = temp - new_temp1 + neigh_temp;

        bf.temperature_field.entry(p).and_modify(|x| *x = new_temp1);
        bf.temperature_field
            .entry(neigh)
            .and_modify(|x| *x = new_temp2);
    }
}