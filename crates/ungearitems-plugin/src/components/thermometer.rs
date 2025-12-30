use crate::metrics;

use bevy::prelude::*;
use rand::Rng;
use uncore_board::behavior::Behavior;
use uncore_board::resources::board_data::BoardData;
use uncore_board::resources::roomdb::RoomDB;
use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};
use uncore_foundation::random_seed;
use uncore_foundation::types::evidence::Evidence;
use uncore_foundation::types::gear::GearSpriteID;
use uncore_foundation::{celsius_to_kelvin, kelvin_to_celsius};
use undifficulty_core::CurrentDifficulty;
use ungear::gear_stuff::GearStuff;
use ungear::types::gear::utils::on_off;
pub use ungearitems_core::components::thermometer::Thermometer;
use unghost_core::HauntState;
use unghost_core::components::GhostSprite;
use unmetrics_core::SendMetric;
use unspatial_core::{BoardPosition, Position};

pub fn update_thermometer(
    mut q_thermometer: Query<(
        &mut Thermometer,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &mut Battery,
        &Electronic,
        &Position,
        &ItemName,
    )>,
    mut gs: GearStuff,
) {
    for (mut thermometer, mut status, mut sprite, toggle, mut battery, electronic, pos, name) in
        q_thermometer.iter_mut()
    {
        let mut rng = random_seed::rng();
        thermometer.frame_counter = thermometer.frame_counter.wrapping_add(1);

        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        // Update Sprite
        sprite.0 = match toggle.is_on {
            true => GearSpriteID::ThermometerOn,
            false => GearSpriteID::ThermometerOff,
        };

        // Update Logic
        if toggle.is_on {
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
            let n = thermometer.frame_counter as usize % thermometer.temp_l2.len();
            thermometer.temp_l2[n] =
                (thermometer.temp_l2[n] * air_mass + thermometer.temp_l1) / (air_mass + 1.0);
            thermometer.temp_l1 =
                (thermometer.temp_l1 * air_mass + temp_reading) / (air_mass + 1.0);
            if thermometer.frame_counter % 5 == 0 {
                let sum_temp: f32 = thermometer.temp_l2.iter().sum();
                let avg_temp: f32 = sum_temp / thermometer.temp_l2.len() as f32;
                thermometer.temp = (avg_temp * 5.0).round() / 5.0;

                // Update blinking_hint_active
                const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                if kelvin_to_celsius(thermometer.temp) < 0.0 && electronic.glitch_timer <= 0.0 {
                    let count = gs
                        .player_profile
                        .times_evidence_acknowledged_on_gear
                        .get(&Evidence::FreezingTemp)
                        .copied()
                        .unwrap_or(0);
                    thermometer.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
                } else {
                    thermometer.blinking_hint_active = false;
                }
            } else {
                // Ensure blinking_hint_active is false if not updating temp this frame,
                // or if we want it to strictly follow the evidence condition.
                // For now, let's ensure it's false if the condition isn't met.
                if !(kelvin_to_celsius(thermometer.temp) < 0.0 && electronic.glitch_timer <= 0.0) {
                    thermometer.blinking_hint_active = false;
                }
            }

            // Possibly play crackling/static sounds during glitches
            if electronic.glitch_timer > 0.0 && random_seed::rng().random_range(0.0..1.0) < 0.3 {
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, &pos);
            }
        }

        // Update Status Text
        let on_s = on_off(toggle.is_on);

        // Show garbled text when glitching
        if toggle.is_on && electronic.glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Temperature: ERR0R",
                1 => "Temperature: ---.--°C",
                2 => "Temperature: ?**.??°C",
                _ => "SENSOR MALFUNCTION",
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, garbled);
            continue;
        }

        // Regular display
        let msg = if toggle.is_on {
            let temp_celsius = kelvin_to_celsius(thermometer.temp);
            if thermometer.blinking_hint_active {
                let temp_str = format!("{:>5.1}ºC", temp_celsius);
                let blinking_temp_str = if thermometer.frame_counter % 30 < 15 {
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
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);
    }
}

fn temperature_update(
    mut bf: ResMut<BoardData>,
    haunt_state: Res<HauntState>,
    roomdb: Res<RoomDB>,
    qt: Query<(&Position, &Behavior)>,
    qg: Query<(&GhostSprite, &Position)>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    let measure = metrics::TEMPERATURE_UPDATE.time_measure();
    let freezing = haunt_state.ghost_dynamics.freezing_temp_clarity;

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
    app.add_systems(Update, (temperature_update, update_thermometer));
}
