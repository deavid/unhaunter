use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use crate::{board::Position, difficulty::CurrentDifficulty};
use bevy::prelude::*;
use rand::Rng;
use uncore::{
    resources::boarddata::BoardData,
    types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition},
};

#[derive(Component, Debug, Clone)]
pub struct Thermometer {
    pub enabled: bool,
    pub temp: f32,
    pub temp_l2: [f32; 5],
    pub temp_l1: f32,
    pub frame_counter: u16,
}

impl Default for Thermometer {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            temp: 10.0,
            temp_l2: [10.0; 5],
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

    fn get_description(&self) -> &'static str {
        "Reads the temperature of the room. Most paranormal interactions have been correlated with unusual cold temperatures."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            format!("Temperature: {:>5.1}ºC", self.temp)
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        // TODO: Add two thresholds: LO: -0.1 and HI: 5.1, with sound effects to notify +
        // distintive icons.
        let mut rng = rand::thread_rng();
        self.frame_counter += 1;
        self.frame_counter %= 65413;
        const K: f32 = 0.7;
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
        let air_mass: f32 = 5.0 / gs.difficulty.0.equipment_sensitivity;

        // Double noise reduction to remove any noise from measurement.
        let n = self.frame_counter as usize % self.temp_l2.len();
        self.temp_l2[n] = (self.temp_l2[n] * air_mass + self.temp_l1) / (air_mass + 1.0);
        self.temp_l1 = (self.temp_l1 * air_mass + temp_reading) / (air_mass + 1.0);
        if self.frame_counter % 5 == 0 {
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
        Gear::new_from_kind(GearKind::Thermometer(value.box_clone()))
    }
}

pub fn temperature_update(
    mut bf: ResMut<BoardData>,
    roomdb: Res<crate::board::RoomDB>,
    qt: Query<(&Position, &crate::uncore_behavior::Behavior)>,
    qg: Query<(&crate::ghost::GhostSprite, &Position)>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    let ambient = bf.ambient_temp;
    for (pos, bh) in qt.iter() {
        let h_out = bh.temp_heat_output();
        if h_out < 0.001 {
            continue;
        }
        let bpos = pos.to_board_position();
        let prev_temp = bf.temperature_field.get(&bpos).copied().unwrap_or(ambient);
        let k = (f32::tanh((19.0 - prev_temp) / 5.0) + 1.0) / 2.0;
        let t_out = h_out * k * 0.5 * difficulty.0.light_heat;
        bf.temperature_field.entry(bpos).and_modify(|t| *t += t_out);
    }
    for (gs, pos) in qg.iter() {
        let bpos = pos.to_board_position();
        let freezing = gs.class.evidences().contains(&Evidence::FreezingTemp);
        let ghost_target_temp: f32 = if freezing { -5.0 } else { 1.0 };
        const GHOST_MAX_POWER: f32 = 0.0002;
        const BREACH_MAX_POWER: f32 = 0.2;
        for npos in bpos.xy_neighbors(1) {
            bf.temperature_field.entry(npos).and_modify(|t| {
                *t = (*t + ghost_target_temp * GHOST_MAX_POWER) / (1.0 + GHOST_MAX_POWER)
            });
        }
        for npos in gs.spawn_point.xy_neighbors(2) {
            bf.temperature_field.entry(npos).and_modify(|t| {
                *t = (*t + ghost_target_temp * BREACH_MAX_POWER) / (1.0 + BREACH_MAX_POWER)
            });
        }
    }

    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let mut rng = SmallRng::from_entropy();
    let old_temps: Vec<(_, _)> = bf
        .temperature_field
        .iter()
        .filter_map(|(p, t)| {
            if rng.gen_range(0..8) == 0 {
                Some((p.clone(), *t))
            } else {
                None
            }
        })
        .collect();
    const OUTSIDE_CONDUCTIVITY: f32 = 100.0;
    const INSIDE_CONDUCTIVITY: f32 = 50.0;

    // Closed Doors
    const OTHER_CONDUCTIVITY: f32 = 2.0;
    const WALL_CONDUCTIVITY: f32 = 0.1;
    let smooth: f32 = 4.0 / difficulty.0.temperature_spread_speed;
    for (p, temp) in old_temps.into_iter() {
        let free = bf
            .collision_field
            .get(&p)
            .map(|x| (x.player_free, x.ghost_free))
            .unwrap_or((true, true));
        let mut self_k = match free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let is_outside = roomdb.room_tiles.get(&p).is_none();
        if is_outside {
            self_k = OUTSIDE_CONDUCTIVITY;
        }

        // let neighbors = p.xy_neighbors(1);
        let neighbors = [p.left(), p.right(), p.top(), p.bottom()];
        let n_idx = rng.gen_range(0..neighbors.len());
        let neigh = neighbors[n_idx].clone();
        let neigh_free = bf
            .collision_field
            .get(&neigh)
            .map(|x| (x.player_free, x.ghost_free))
            .unwrap_or((true, true));
        let neigh_k = match neigh_free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let nis_outside = roomdb.room_tiles.get(&neigh).is_none();
        if nis_outside {
            self_k = OUTSIDE_CONDUCTIVITY;
        }
        let neigh_temp = bf.temperature_field.get(&neigh).copied().unwrap_or(ambient);
        let mid_temp = (temp * self_k + neigh_temp * neigh_k) / (self_k + neigh_k);
        let conductivity = (self_k.recip() + neigh_k.recip()).recip() / smooth;
        let new_temp1 = (temp + mid_temp * conductivity) / (conductivity + 1.0);
        let new_temp2 = temp - new_temp1 + neigh_temp;
        bf.temperature_field.entry(p).and_modify(|x| *x = new_temp1);
        bf.temperature_field
            .entry(neigh)
            .and_modify(|x| *x = new_temp2);
    }
}
