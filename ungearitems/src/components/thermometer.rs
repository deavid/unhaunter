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
        if self.frame_counter.is_multiple_of(5) {
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
        const GHOST_MAX_POWER: f32 = 0.01;
        const BREACH_MAX_POWER: f32 = 10.0;
        let ghost_in_room = roomdb.room_tiles.get(&bpos);
        let breach_in_room = roomdb.room_tiles.get(&gs.spawn_point);
        let power = freezing * 0.5 + 0.5;
        const ENABLE_GHOST_COLD_TEMPS: bool = true;
        if ENABLE_GHOST_COLD_TEMPS {
            for npos in bpos.iter_xy_neighbors(3, bf.map_size) {
                if ghost_in_room != roomdb.room_tiles.get(&npos)
                    || !bf.collision_field[npos.ndidx()].player_free
                {
                    // Only make current room colder
                    continue;
                }

                // Calculate distance-based power decay
                let distance2 = npos.distance2(&bpos) + 1.0; // Add small constant to avoid division by zero
                let distance_decay = 1.0 / distance2;
                let effective_power = GHOST_MAX_POWER * power * distance_decay;

                let t = &mut bf.temperature_field[npos.ndidx()];
                *t = (*t + ghost_target_temp * effective_power) / (1.0 + effective_power);
            }
        }
        for npos in gs.spawn_point.iter_xy_neighbors(3, bf.map_size) {
            if breach_in_room != roomdb.room_tiles.get(&npos)
                || !bf.collision_field[npos.ndidx()].player_free
            {
                // Only make current room colder
                continue;
            }

            // Calculate distance-based power decay
            let distance2 = npos.distance2(&gs.spawn_point) + 1.0; // Add small constant to avoid division by zero
            let distance_decay = 1.0 / distance2;
            let effective_power = BREACH_MAX_POWER * power * distance_decay;

            let t = &mut bf.temperature_field[npos.ndidx()];
            *t = (*t + ghost_target_temp * effective_power) / (1.0 + effective_power);
        }
    }

    let mut rng = random_seed::rng();
    let old_temps: Vec<(_, _)> = bf
        .temperature_field
        .indexed_iter()
        .filter_map(|(p, t)| {
            // Use temperature activity (how much this tile changed last frame) for selection
            let activity = bf.temperature_activity.get(p).copied().unwrap_or(0.0);

            // Base selection probability: higher activity = higher chance
            // Scale activity to reasonable range (0.0-1.0+) and add baseline
            let activity_factor = (activity * 0.02).clamp(0.0, 1.0) + 0.001;

            if rng.random_range(0.0..1.0) < activity_factor {
                Some((p, *t))
            } else {
                None
            }
        })
        .collect();
    const OUTSIDE_CONDUCTIVITY: f32 = 1000000.0;
    const INSIDE_CONDUCTIVITY: f32 = 80000.0;

    // Closed Doors
    const OTHER_CONDUCTIVITY: f32 = 20000.0;
    const WALL_CONDUCTIVITY: f32 = 0.00001;
    let smooth: f32 = 1.0; // / difficulty.0.temperature_spread_speed;

    // Collect all energy changes before applying them
    let mut energy_changes: std::collections::HashMap<(usize, usize, usize), Vec<f32>> =
        std::collections::HashMap::new();

    for (p, temp) in old_temps.into_iter() {
        let cp = &bf.collision_field[p];
        let free = (cp.see_through, cp.see_through || cp.is_dynamic);

        let mut self_k = match free {
            (true, true) => INSIDE_CONDUCTIVITY,
            (false, false) => WALL_CONDUCTIVITY,
            _ => OTHER_CONDUCTIVITY,
        };
        let bpos = BoardPosition::from_ndidx(p);
        let is_outside = roomdb.room_tiles.get(&bpos).is_none();
        if is_outside && cp.see_through {
            self_k = OUTSIDE_CONDUCTIVITY;
        }

        // Check if this is a stair tile - if so, add vertical neighbor
        let mut neighbors = vec![bpos.left(), bpos.right(), bpos.top(), bpos.bottom()];

        // Add stair connection as a neighbor with very high priority
        if cp.stair_offset != 0 {
            let stair_target_z = bpos.z + cp.stair_offset as i64;
            if stair_target_z >= 0 && stair_target_z < bf.map_size.2 as i64 {
                let stair_neighbor = BoardPosition {
                    x: bpos.x,
                    y: bpos.y,
                    z: stair_target_z,
                };
                // Add stair neighbors - we'll process all neighbors now instead of random selection
                neighbors.push(stair_neighbor.left());
                neighbors.push(stair_neighbor.right());
                neighbors.push(stair_neighbor.top());
                neighbors.push(stair_neighbor.bottom());
            }
        }

        // Process only 1 neighbor at random
        // use rand::prelude::IndexedRandom;
        // if let Some(neigh) = neighbors.choose(&mut rng) {
        for neigh in &neighbors {
            let neigh_ndidx = neigh.ndidx();
            let Some(neigh_free) = bf
                .collision_field
                .get(neigh_ndidx)
                .map(|ncp| (ncp.see_through, ncp.see_through || ncp.is_dynamic))
            else {
                continue;
            };

            // Check if this is a stair connection
            let is_stair_connection = neigh.z != bpos.z;

            let mut neigh_k = match neigh_free {
                (true, true) => INSIDE_CONDUCTIVITY,
                (false, false) => WALL_CONDUCTIVITY,
                _ => OTHER_CONDUCTIVITY,
            };

            let nis_outside = roomdb.room_tiles.get(neigh).is_none();
            if nis_outside && neigh_free.0 && !is_stair_connection {
                neigh_k = OUTSIDE_CONDUCTIVITY;
            }
            let neigh_temp = bf
                .temperature_field
                .get(neigh_ndidx)
                .copied()
                .unwrap_or(bf.ambient_temp);

            // Convert temperatures to energy using E = T³ (energy conservation approach)
            // No temperature floor restriction - work directly with actual temperatures
            let temp_energy = temp.powi(3);
            let neigh_energy = neigh_temp.powi(3);

            // Use the actual energies for diffusion calculations
            let temp_energy_for_diffusion = temp_energy;
            let neigh_energy_for_diffusion = neigh_energy;

            let self_thermal_mass = match free {
                (true, true) => 0.9,
                (false, false) => 0.00001,
                _ => 1.0,
            };

            let neigh_thermal_mass = match neigh_free {
                (true, true) => 0.9,
                (false, false) => 0.00001,
                _ => 1.0,
            };

            // Calculate weighted average energy, accounting for thermal mass
            // Use diffusion-limited energies for the energy transfer calculation
            let total_mass = self_thermal_mass + neigh_thermal_mass;
            let mid_energy = (temp_energy_for_diffusion * self_thermal_mass
                + neigh_energy_for_diffusion * neigh_thermal_mass)
                / total_mass;

            let conductivity = (self_k.recip() + neigh_k.recip()).recip() / smooth;
            let energy_diff = (temp_energy_for_diffusion + mid_energy * conductivity)
                / (conductivity + 1.0)
                - temp_energy_for_diffusion;

            // Apply stability limit: cap energy changes to 10% of each component's energy
            const MAX_ENERGY_CHANGE_RATIO: f32 = 0.9;
            let max_self_energy_change = temp_energy_for_diffusion * MAX_ENERGY_CHANGE_RATIO;
            let max_neigh_energy_change = neigh_energy_for_diffusion * MAX_ENERGY_CHANGE_RATIO;

            // Limit the base energy difference to prevent instability
            let limited_energy_diff = energy_diff.clamp(
                -max_self_energy_change.min(max_neigh_energy_change),
                max_self_energy_change.min(max_neigh_energy_change),
            );

            // Apply energy diffusion with thermal mass consideration
            // Walls (low thermal mass) change temperature more easily
            let self_energy_change = limited_energy_diff / self_thermal_mass;
            let neigh_energy_change = -limited_energy_diff / neigh_thermal_mass;

            // Apply changes to the actual energies (not diffusion-limited ones)
            // Remove 1°C temperature floor restriction for energy calculations
            let new_energy1 = temp_energy + self_energy_change;
            let new_energy2 = neigh_energy + neigh_energy_change;

            // Handle ambient temperature influence in energy space
            let adjusted_energy1 = if is_outside && nis_outside {
                let k: f32 = 0.02;
                let ambient_energy = bf.ambient_temp.powi(3);
                (new_energy1 + ambient_energy * k) / (1.00 + k)
            } else {
                new_energy1
            };

            let adjusted_energy2 = if is_outside && nis_outside {
                let k: f32 = 0.02;
                let ambient_energy = bf.ambient_temp.powi(3);
                (new_energy2 + ambient_energy * k) / (1.00 + k)
            } else {
                new_energy2
            };

            // Store energy changes instead of temperature changes
            energy_changes.entry(p).or_default().push(adjusted_energy1);
            energy_changes
                .entry(neigh_ndidx)
                .or_default()
                .push(adjusted_energy2);
        }
    }

    // Apply all accumulated energy changes by averaging them in energy space
    // and track activity for adaptive selection
    let mut updated_positions = std::collections::HashSet::new();
    let mut debug_total_activity = 0.0;
    let mut debug_activity_count = 0;
    for (pos_idx, energy_list) in energy_changes {
        if !energy_list.is_empty() {
            let old_temp = bf.temperature_field[pos_idx];

            // Average all energies in energy space
            let avg_energy = energy_list.iter().sum::<f32>() / energy_list.len() as f32;

            // Check for NaN and clamp energy to reasonable bounds
            if avg_energy.is_finite() && avg_energy > 0.0 {
                // Convert back to temperature using T = ∛E
                let new_temp = avg_energy
                    .cbrt()
                    .clamp(celsius_to_kelvin(-50.0), celsius_to_kelvin(100.0));
                bf.temperature_field[pos_idx] = new_temp;

                // Update activity: track the sum of absolute temperature changes for this tile
                // Convert energy changes back to temperature space for reasonable activity values
                let mut total_temp_change = 0.0;
                for energy in &energy_list {
                    let temp_from_energy = energy.cbrt();
                    total_temp_change += (temp_from_energy - old_temp).abs();
                }

                // Debug tracking
                debug_total_activity += total_temp_change;
                debug_activity_count += 1;

                let current_activity = bf.temperature_activity.get(pos_idx).copied().unwrap_or(0.0);
                // Exponential moving average with activity addition
                let new_activity = (current_activity / 1.05) + total_temp_change;
                bf.temperature_activity[pos_idx] = new_activity;

                updated_positions.insert(pos_idx);
            }
        }
    }

    // Debug output for activity tracking
    if debug_activity_count > 0 {
        let avg_activity = debug_total_activity / debug_activity_count as f32;

        // Calculate average activity across all tiles
        let total_tiles = bf.temperature_activity.len();
        let sum_activity: f32 = bf.temperature_activity.iter().sum();
        let avg_tile_activity = if total_tiles > 0 {
            sum_activity / total_tiles as f32
        } else {
            0.0
        };

        debug!(
            "Frame activity debug: {} tiles updated, avg total_temp_change: {:.6}, avg_tile_activity: {:.6}",
            debug_activity_count, avg_activity, avg_tile_activity
        );
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, temperature_update);
}
