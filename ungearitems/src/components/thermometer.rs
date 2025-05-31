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
use uncore::random_seed;
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
    pub display_glitch_timer: f32,
    pub blinking_hint_active: bool,
}

impl Default for Thermometer {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            temp: celsius_to_kelvin(10.0),
            temp_l2: [celsius_to_kelvin(10.0); 5],
            temp_l1: celsius_to_kelvin(10.0),
            frame_counter: Default::default(),
            display_glitch_timer: Default::default(),
            blinking_hint_active: false,
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

        // Show garbled text when glitching
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Temperature: ERR0R",
                1 => "Temperature: ---.--°C",
                2 => "Temperature: ?**.??°C",
                _ => "SENSOR MALFUNCTION",
            };
            return format!("{name}: {on_s}\n{garbled}");
        }

        // Regular display
        let msg = if self.enabled {
            let temp_celsius = kelvin_to_celsius(self.temp);
            if self.blinking_hint_active {
                let temp_str = format!("{:>5.1}ºC", temp_celsius);
                let blinking_temp_str = if self.frame_counter % 30 < 15 {
                    format!(">[{}]<", temp_str.trim())
                } else {
                    format!("  {}  ", temp_str.trim())
                };
                format!("Temperature: {}", blinking_temp_str)
            } else {
                format!("Temperature: {:>5.1}ºC", temp_celsius)
            }
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        // TODO: Add two thresholds: LO: -0.1 and HI: 5.1, with sound effects to notify +
        // distintive icons.
        let mut rng = random_seed::rng();
        self.frame_counter += 1;
        self.frame_counter %= 65413;
        const K: f32 = 0.7;
        let pos = Position {
            x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
            y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
            z: pos.z,
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

            // Update blinking_hint_active
            const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
            if kelvin_to_celsius(self.temp) < 0.0 && self.display_glitch_timer <= 0.0 {
                let count = gs
                    .player_profile
                    .times_evidence_acknowledged_on_gear
                    .get(&Evidence::FreezingTemp)
                    .copied()
                    .unwrap_or(0);
                self.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
            } else {
                self.blinking_hint_active = false;
            }
        } else {
            // Ensure blinking_hint_active is false if not updating temp this frame,
            // or if we want it to strictly follow the evidence condition.
            // For now, let's ensure it's false if the condition isn't met.
            if !(kelvin_to_celsius(self.temp) < 0.0 && self.display_glitch_timer <= 0.0) {
                self.blinking_hint_active = false;
            }
        }

        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();

            // Possibly play crackling/static sounds during glitches
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.3 {
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, &pos);
            }
        }

        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn is_electronic(&self) -> bool {
        true
    }

    fn apply_electromagnetic_interference(&mut self, warning_level: f32, distance2: f32) {
        if warning_level < 0.0001 || !self.enabled {
            return;
        }
        let mut rng = random_seed::rng();

        // Scale effect by distance and warning level
        let effect_strength = warning_level * (100.0 / distance2).min(1.0);

        // Random temperature spikes
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
            // Random temperature spike - show extreme cold or hot temperatures
            if rng.random_bool(0.7) {
                // Show extremely cold temperatures
                self.temp = celsius_to_kelvin(rng.random_range(-20.0..-5.0));
            } else {
                // Show extremely hot temperatures
                self.temp = celsius_to_kelvin(rng.random_range(30.0..60.0));
            }

            // Add a display glitch timer field to Thermometer struct
            self.display_glitch_timer = 0.3;
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn can_enable(&self) -> bool {
        true // Thermometer can always be toggled
    }

    fn is_status_text_showing_evidence(&self) -> f32 {
        if self.is_enabled()
            && self.display_glitch_timer <= 0.0
            && kelvin_to_celsius(self.temp) < 0.0
        {
            1.0
        } else {
            0.0
        }
    }

    fn is_blinking_hint_active(&self) -> bool {
        self.blinking_hint_active
    }
}

impl From<Thermometer> for Gear {
    fn from(value: Thermometer) -> Self {
        Gear::new_from_kind(GearKind::Thermometer, value.box_clone())
    }
}

fn temperature_update(
    mut bf: ResMut<BoardData>,
    roomdb: Res<RoomDB>,
    qt: Query<(&Position, &Behavior)>,
    qg: Query<(&GhostSprite, &Position)>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    let measure = metrics::TEMPERATURE_UPDATE.time_measure();
    let freezing = bf.ghost_dynamics.freezing_temp_clarity;

    for (pos, bh) in qt.iter() {
        let h_out = bh.temp_heat_output();
        if h_out < 0.001 {
            continue;
        }
        let bpos = pos.to_board_position();
        let prev_temp = bf.temperature_field[bpos.ndidx()];
        let k = (f32::tanh((19.0 - prev_temp) / 5.0) + 1.0) / 2.0;
        let t_out = h_out * k * 0.2 * difficulty.0.light_heat;
        bf.temperature_field[bpos.ndidx()] += t_out;
    }
    for (gs, pos) in qg.iter() {
        let bpos = pos.to_board_position();
        if bpos.z < 0 || bpos.z >= bf.map_size.2 as i64 {
            continue;
        }
        let ghost_target_temp: f32 = celsius_to_kelvin(1.0 - 4.0 * freezing);
        const GHOST_MAX_POWER: f32 = 1.0;
        const BREACH_MAX_POWER: f32 = 20.0;
        let ghost_in_room = roomdb.room_tiles.get(&bpos);
        let breach_in_room = roomdb.room_tiles.get(&gs.spawn_point);
        let power = freezing * 0.5 + 0.5;
        for npos in bpos.iter_xy_neighbors(3, bf.map_size) {
            if ghost_in_room != roomdb.room_tiles.get(&npos)
                || !bf.collision_field[npos.ndidx()].player_free
            {
                // Only make current room colder
                continue;
            }
            let t = &mut bf.temperature_field[npos.ndidx()];
            *t = (*t + ghost_target_temp * GHOST_MAX_POWER * power)
                / (1.0 + GHOST_MAX_POWER * power);
        }
        for npos in gs.spawn_point.iter_xy_neighbors(3, bf.map_size) {
            if breach_in_room != roomdb.room_tiles.get(&gs.spawn_point)
                || !bf.collision_field[npos.ndidx()].player_free
            {
                // Only make current room colder
                continue;
            }
            let t = &mut bf.temperature_field[npos.ndidx()];
            *t = (*t + ghost_target_temp * BREACH_MAX_POWER * power)
                / (1.0 + BREACH_MAX_POWER * power)
        }
    }

    let mut rng = random_seed::rng();
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
    const OUTSIDE_CONDUCTIVITY: f32 = 1000000.0;
    const INSIDE_CONDUCTIVITY: f32 = 80000.0;

    // Closed Doors
    const OTHER_CONDUCTIVITY: f32 = 2000.0;
    const WALL_CONDUCTIVITY: f32 = 0.00001;
    let smooth: f32 = 1.00 / difficulty.0.temperature_spread_speed;
    for (p, temp) in old_temps.into_iter() {
        let cp = &bf.collision_field[p];
        let free = (cp.player_free, cp.player_free || cp.is_dynamic);

        let mut self_k = match free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let bpos = BoardPosition::from_ndidx(p);
        let is_outside = roomdb.room_tiles.get(&bpos).is_none();
        if is_outside && cp.player_free {
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
            .map(|ncp| (ncp.player_free, cp.player_free || cp.is_dynamic))
        else {
            continue;
        };

        let neigh_k = match neigh_free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let nis_outside = roomdb.room_tiles.get(&neigh).is_none();
        if nis_outside && neigh_free.0 {
            self_k = OUTSIDE_CONDUCTIVITY;
        }
        let neigh_temp = bf
            .temperature_field
            .get(neigh_ndidx)
            .copied()
            .unwrap_or(bf.ambient_temp);
        let mid_temp = (temp * self_k.min(10.0) + neigh_temp * neigh_k.min(10.0))
            / (self_k.min(10.0) + neigh_k.min(10.0));
        let conductivity = (self_k.recip() + neigh_k.recip()).recip() / smooth;
        let diff = (temp + mid_temp * conductivity) / (conductivity + 1.0) - temp;
        let mut new_temp1: f32;
        let mut new_temp2: f32;
        // Break conservation of energy to make a tendency of temps to go cold (by not going warm)
        const COLD_EFFECT: f32 = 0.3;
        let filter_cold_effect = kelvin_to_celsius(mid_temp - 5.0).clamp(0.0, 20.0) / 20.0;
        let cold: f32 =
            1.0 + COLD_EFFECT * filter_cold_effect.powi(2) * if is_outside { 0.0 } else { 1.0 };
        if diff > 0.0 {
            new_temp1 = temp + diff / cold;
            new_temp2 = neigh_temp - diff;
        } else {
            new_temp1 = temp + diff;
            new_temp2 = neigh_temp - diff / cold;
        }

        if is_outside || nis_outside {
            let k: f32 = 0.2;
            new_temp1 = (new_temp1 + bf.ambient_temp * k) / (1.00 + k);
            new_temp2 = (new_temp2 + bf.ambient_temp * k) / (1.00 + k);
        }

        bf.temperature_field[p] = new_temp1;
        bf.temperature_field[neigh_ndidx] = new_temp2;
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, temperature_update);
}
