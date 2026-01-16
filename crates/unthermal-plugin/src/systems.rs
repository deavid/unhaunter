use crate::metrics;
use bevy::prelude::*;
use rand::Rng;
use unbehavior::behavior::Behavior;
use unboard_core::resources::board_topology::BoardTopology;
use unboard_core::resources::roomdb::RoomDB;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::loadlevel::{LevelReadyEvent, MapGeometryInitializedEvent};
use unfoundation_core::random_seed;
use unfoundation_core::utils::temperature::celsius_to_kelvin;
use unghost_core::components::GhostSprite;
use unghost_core::resources::haunt_state::HauntState;
use unmetrics_core::metrics::SendMetric;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;
use unthermal_core::resources::ThermalGrid;

pub fn temperature_update(
    mut thermal_grid: ResMut<ThermalGrid>,
    bf: Res<BoardTopology>,
    haunt_state: Res<HauntState>,
    roomdb: Res<RoomDB>,
    qt: Query<(&Position, &Behavior)>,
    qg: Query<(&GhostSprite, &Position)>,
    difficulty: Res<CurrentDifficulty>,
) {
    let measure = metrics::TEMPERATURE_UPDATE.time_measure();
    let freezing = haunt_state.ghost_dynamics.freezing_temp_clarity;

    for (pos, bh) in qt.iter() {
        let h_out: f32 = bh.temp_heat_output();
        if h_out < 0.001 {
            continue;
        }
        let bpos: BoardPosition = pos.to_board_position();
        let prev_temp = thermal_grid.temperature_field[bpos.ndidx()];
        let k = (f32::tanh((19.0 - prev_temp) / 5.0) + 1.0) / 2.0;
        let t_out = h_out * k * 0.2 * difficulty.0.light_heat;
        thermal_grid.temperature_field[bpos.ndidx()] += t_out;
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
                    continue;
                }

                let distance2 = npos.distance2(&bpos) + 1.0;
                let distance_decay = 1.0 / distance2;
                let effective_power = GHOST_MAX_POWER * power * distance_decay;

                let t = &mut thermal_grid.temperature_field[npos.ndidx()];
                *t = (*t + ghost_target_temp * effective_power) / (1.0 + effective_power);
            }
        }
        for npos in gs.spawn_point.iter_xy_neighbors(3, bf.map_size) {
            if breach_in_room != roomdb.room_tiles.get(&npos)
                || !bf.collision_field[npos.ndidx()].player_free
            {
                continue;
            }

            let distance2 = npos.distance2(&gs.spawn_point) + 1.0;
            let distance_decay = 1.0 / distance2;
            let effective_power = BREACH_MAX_POWER * power * distance_decay;

            let t = &mut thermal_grid.temperature_field[npos.ndidx()];
            *t = (*t + ghost_target_temp * effective_power) / (1.0 + effective_power);
        }
    }

    let mut rng = random_seed::rng();
    let old_temps: Vec<(_, _)> = thermal_grid
        .temperature_field
        .indexed_iter()
        .filter_map(|(p, t)| {
            let activity = thermal_grid
                .temperature_activity
                .get(p)
                .copied()
                .unwrap_or(0.0);
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
    const OTHER_CONDUCTIVITY: f32 = 20000.0;
    const WALL_CONDUCTIVITY: f32 = 0.00001;
    let smooth: f32 = 1.0;

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

        let mut neighbors = vec![bpos.left(), bpos.right(), bpos.top(), bpos.bottom()];

        if cp.stair_offset != 0 {
            let stair_target_z = bpos.z + cp.stair_offset as i64;
            if stair_target_z >= 0 && stair_target_z < bf.map_size.2 as i64 {
                let stair_neighbor = BoardPosition {
                    x: bpos.x,
                    y: bpos.y,
                    z: stair_target_z,
                };
                neighbors.push(stair_neighbor.left());
                neighbors.push(stair_neighbor.right());
                neighbors.push(stair_neighbor.top());
                neighbors.push(stair_neighbor.bottom());
            }
        }

        for neigh in &neighbors {
            let neigh_ndidx = neigh.ndidx();
            let Some(neigh_free) = bf
                .collision_field
                .get(neigh_ndidx)
                .map(|ncp| (ncp.see_through, ncp.see_through || ncp.is_dynamic))
            else {
                continue;
            };

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
            let neigh_temp = thermal_grid
                .temperature_field
                .get(neigh_ndidx)
                .copied()
                .unwrap_or(thermal_grid.ambient_temp);

            let temp_energy = temp.powi(3);
            let neigh_energy = neigh_temp.powi(3);

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

            let total_mass = self_thermal_mass + neigh_thermal_mass;
            let mid_energy =
                (temp_energy * self_thermal_mass + neigh_energy * neigh_thermal_mass) / total_mass;

            let conductivity = (self_k.recip() + neigh_k.recip()).recip() / smooth;
            let energy_diff =
                (temp_energy + mid_energy * conductivity) / (conductivity + 1.0) - temp_energy;

            const MAX_ENERGY_CHANGE_RATIO: f32 = 0.9;
            let max_self_energy_change = temp_energy * MAX_ENERGY_CHANGE_RATIO;
            let max_neigh_energy_change = neigh_energy * MAX_ENERGY_CHANGE_RATIO;

            let limited_energy_diff = energy_diff.clamp(
                -max_self_energy_change.min(max_neigh_energy_change),
                max_self_energy_change.min(max_neigh_energy_change),
            );

            let self_energy_change = limited_energy_diff / self_thermal_mass;
            let neigh_energy_change = -limited_energy_diff / neigh_thermal_mass;

            let new_energy1 = temp_energy + self_energy_change;
            let new_energy2 = neigh_energy + neigh_energy_change;

            let adjusted_energy1 = if is_outside && nis_outside {
                let k: f32 = 0.02;
                let ambient_energy = thermal_grid.ambient_temp.powi(3);
                (new_energy1 + ambient_energy * k) / (1.00 + k)
            } else {
                new_energy1
            };

            let adjusted_energy2 = if is_outside && nis_outside {
                let k: f32 = 0.02;
                let ambient_energy = thermal_grid.ambient_temp.powi(3);
                (new_energy2 + ambient_energy * k) / (1.00 + k)
            } else {
                new_energy2
            };

            energy_changes.entry(p).or_default().push(adjusted_energy1);
            energy_changes
                .entry(neigh_ndidx)
                .or_default()
                .push(adjusted_energy2);
        }
    }

    for (pos_idx, energy_list) in energy_changes {
        if !energy_list.is_empty() {
            let old_temp = thermal_grid.temperature_field[pos_idx];
            let avg_energy = energy_list.iter().sum::<f32>() / energy_list.len() as f32;

            if avg_energy.is_finite() && avg_energy > 0.0 {
                let new_temp = avg_energy
                    .cbrt()
                    .clamp(celsius_to_kelvin(-50.0), celsius_to_kelvin(100.0));
                thermal_grid.temperature_field[pos_idx] = new_temp;

                let mut total_temp_change = 0.0;
                for energy in &energy_list {
                    total_temp_change += (energy.cbrt() - old_temp).abs();
                }

                let current_activity = thermal_grid
                    .temperature_activity
                    .get(pos_idx)
                    .copied()
                    .unwrap_or(0.0);
                let new_activity = (current_activity / 1.05) + total_temp_change;
                thermal_grid.temperature_activity[pos_idx] = new_activity;
            }
        }
    }

    measure.end_ms();
}

pub fn init_thermal_grid_allocation(
    mut thermal_grid: ResMut<ThermalGrid>,
    bf: Res<BoardTopology>,
    mut ev: MessageReader<MapGeometryInitializedEvent>,
) {
    for ev in ev.read() {
        use ndarray::Array3;
        thermal_grid.temperature_field = Array3::from_elem(ev.map_size, bf.ambient_temp);
        thermal_grid.temperature_field_prev = Array3::from_elem(ev.map_size, bf.ambient_temp);
        thermal_grid.temperature_activity = Array3::from_elem(ev.map_size, 0.0);
        thermal_grid.connectivity_scores = Array3::from_elem(
            ev.map_size,
            thermal_grid.temp_diffusion_config.default_score,
        );
        thermal_grid.ambient_temp = bf.ambient_temp;
    }
}

pub fn init_thermal_grid_content(
    mut thermal_grid: ResMut<ThermalGrid>,
    bf: Res<BoardTopology>,
    haunt_state: Res<HauntState>,
    roomdb: Res<RoomDB>,
    mut ev: MessageReader<LevelReadyEvent>,
) {
    if ev.is_empty() {
        return;
    }
    ev.clear();

    let mut rng = random_seed::rng();
    let ambient_temp = thermal_grid.ambient_temp;
    let breach_room = roomdb
        .room_tiles
        .get(&haunt_state.breach_pos.to_board_position());

    for (idxpos, temperature) in thermal_grid.temperature_field.indexed_iter_mut() {
        let room = roomdb.room_tiles.get(&BoardPosition::from_ndidx(idxpos));
        if room == breach_room {
            *temperature = celsius_to_kelvin(0.5);
        } else {
            let ambient = ambient_temp + rng.random_range(-3.0..3.0);
            *temperature = ambient;
        }
    }

    // Connectivity scores
    let config = thermal_grid.temp_diffusion_config.clone();
    crate::utils::precompute_connectivity_scores(
        bf.map_size,
        &bf.collision_field,
        &mut thermal_grid.connectivity_scores,
        &config,
    );
}
