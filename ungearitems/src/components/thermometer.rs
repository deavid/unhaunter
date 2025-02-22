use crate::metrics;

use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore::behavior::Behavior;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::position::Position;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::metric_recorder::SendMetric;
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;
use uncore::types::evidence::Evidence;
use uncore::types::gear::equipmentposition::EquipmentPosition;
use uncore::{celsius_to_kelvin, kelvin_to_celsius};

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
            temp: celsius_to_kelvin(10.0),
            temp_l2: [celsius_to_kelvin(10.0); 5],
            temp_l1: celsius_to_kelvin(10.0),
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
            format!("Temperature: {:>5.1}ºC", kelvin_to_celsius(self.temp))
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        // TODO: Add two thresholds: LO: -0.1 and HI: 5.1, with sound effects to notify +
        // distintive icons.
        let mut rng = rand::rng();
        self.frame_counter += 1;
        self.frame_counter %= 65413;
        const K: f32 = 0.7;
        let pos = Position {
            x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
            y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
            z: pos.z + rng.random_range(-K..K) + rng.random_range(-K..K),
            global_z: pos.global_z,
        };
        let bpos = pos.to_board_position();
        let temperature = gs.bf.temperature_field[bpos.ndidx()];
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
        Gear::new_from_kind(GearKind::Thermometer, value.box_clone())
    }
}

pub fn temperature_update(
    mut bf: ResMut<BoardData>,
    roomdb: Res<RoomDB>,
    qt: Query<(&Position, &Behavior)>,
    qg: Query<(&GhostSprite, &Position)>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    let measure = metrics::TEMPERATURE_UPDATE.time_measure();

    for (pos, bh) in qt.iter() {
        let h_out = bh.temp_heat_output();
        if h_out < 0.001 {
            continue;
        }
        let bpos = pos.to_board_position();
        let prev_temp = bf.temperature_field[bpos.ndidx()];
        let k = (f32::tanh((19.0 - prev_temp) / 5.0) + 1.0) / 2.0;
        let t_out = h_out * k * 0.5 * difficulty.0.light_heat;
        bf.temperature_field[bpos.ndidx()] += t_out;
    }
    for (gs, pos) in qg.iter() {
        let bpos = pos.to_board_position();
        let freezing = gs.class.evidences().contains(&Evidence::FreezingTemp);
        let ghost_target_temp: f32 = celsius_to_kelvin(if freezing { -5.0 } else { 1.0 });
        const GHOST_MAX_POWER: f32 = 0.0002;
        const BREACH_MAX_POWER: f32 = 0.2;
        for npos in bpos.xy_neighbors(1) {
            let t = &mut bf.temperature_field[npos.ndidx()];
            *t = (*t + ghost_target_temp * GHOST_MAX_POWER) / (1.0 + GHOST_MAX_POWER);
        }
        for npos in gs.spawn_point.xy_neighbors(2) {
            let t = &mut bf.temperature_field[npos.ndidx()];
            *t = (*t + ghost_target_temp * BREACH_MAX_POWER) / (1.0 + BREACH_MAX_POWER)
        }
    }

    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let mut rng = SmallRng::from_os_rng();
    let old_temps: Vec<(_, _)> = bf
        .temperature_field
        .indexed_iter()
        .filter_map(|(p, t)| {
            if rng.random_range(0..8) == 0 {
                Some((p, *t))
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
        let cp = &bf.collision_field[p];
        let free = (cp.player_free, cp.ghost_free);

        let mut self_k = match free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let bpos = BoardPosition::from_ndidx(p);
        let is_outside = roomdb.room_tiles.get(&bpos).is_none();
        if is_outside {
            self_k = OUTSIDE_CONDUCTIVITY;
        }

        // let neighbors = bpos.xy_neighbors(1);
        let neighbors = [bpos.left(), bpos.right(), bpos.top(), bpos.bottom()];
        let n_idx = rng.random_range(0..neighbors.len());
        let neigh = neighbors[n_idx].clone();
        let neigh_ndidx = neigh.ndidx();
        let Some(neigh_free) = bf
            .collision_field
            .get(neigh_ndidx)
            .map(|ncp| (ncp.player_free, ncp.ghost_free))
        else {
            continue;
        };

        let neigh_k = match neigh_free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let nis_outside = roomdb.room_tiles.get(&neigh).is_none();
        if nis_outside {
            self_k = OUTSIDE_CONDUCTIVITY;
        }
        let neigh_temp = bf
            .temperature_field
            .get(neigh_ndidx)
            .copied()
            .unwrap_or(bf.ambient_temp);
        let mid_temp = (temp * self_k + neigh_temp * neigh_k) / (self_k + neigh_k);
        let conductivity = (self_k.recip() + neigh_k.recip()).recip() / smooth;
        let new_temp1 = (temp + mid_temp * conductivity) / (conductivity + 1.0);
        let new_temp2 = temp - new_temp1 + neigh_temp;
        bf.temperature_field[p] = new_temp1;
        bf.temperature_field[neigh_ndidx] = new_temp2;
    }

    measure.end_ms();
}
