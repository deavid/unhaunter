use bevy::prelude::*;
use unboard_core::resources::roomdb::RoomTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unlight_core::resources::light_grid::LightGrid;
use unplayer_core::components::{MainPlayer, PlayerSpectating};
use unfog_core::miasma::MiasmaGrid;
use unsoundfield_core::resources::SoundGrid;
use unspatial_core::position::Position;
use unthermal_core::resources::ThermalGrid;
use untruck_core::components::in_truck::InTruck;
use unvitals_core::components::PlayerVitals;

pub(crate) fn calculate_sanity(crazyness: f32) -> f32 {
    const LINEAR: f32 = 30.0;
    const SCALE: f32 = 100.0;
    (SCALE * LINEAR) / ((crazyness + LINEAR * LINEAR).max(0.01).sqrt())
}

pub(crate) fn drain_sanity_from_environment(
    time: Res<Time>,
    mut qp: Query<
        (&mut PlayerVitals, &Position),
        (
            With<MainPlayer>,
            Without<InTruck>,
            Without<PlayerSpectating>,
        ),
    >,
    thermal_grid: Option<Res<ThermalGrid>>,
    sound_grid: Option<Res<SoundGrid>>,
    lg: Option<Res<LightGrid>>,
    miasma: Option<Res<MiasmaGrid>>,
    room_topology: Res<RoomTopology>,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    let (Some(thermal_grid), Some(sound_grid), Some(lg)) = (thermal_grid, sound_grid, lg) else {
        return;
    };
    for (mut ps, pos) in &mut qp {
        let bpos = pos.to_board_position();
        let p = bpos.ndidx();
        if p.0 >= lg.light_field.shape()[0]
            || p.1 >= lg.light_field.shape()[1]
            || p.2 >= lg.light_field.shape()[2]
        {
            continue;
        }
        let lux = lg.light_field[p].lux.sqrt() + 0.001;
        let temp = thermal_grid.temperature_field[p];
        let f_temp = (temp - thermal_grid.ambient_temp / 2.0).clamp(0.0, 10.0) + 1.0;
        let f_temp2 = (thermal_grid.ambient_temp / 2.0 - temp).clamp(0.0, 10.0) + 1.0;
        let mut sound = 0.0;
        for bpos in bpos.iter_xy_neighbors_nosize(3) {
            sound += sound_grid
                .sound_field
                .get(&bpos)
                .map(|x: &Vec<Vec2>| x.iter().map(|y: &Vec2| y.length()).sum::<f32>())
                .unwrap_or_default()
                * 10.0;
        }
        const MASS: f32 = 10.0;
        if room_topology.room_tiles.contains_key(&bpos) {
            ps.mean_sound =
                ((sound * dt + ps.mean_sound * MASS) / (MASS + dt)).clamp(0.00000001, 100000.0);
        } else {
            // prevent sanity from being lost outside of the location.
            ps.mean_sound /= 1.8_f32.powf(dt);
        }
        let crazy = lux.max(0.00001).recip() / f_temp * f_temp2 * ps.mean_sound * 10.0
            + ps.mean_sound / f_temp * f_temp2;
        let sanity_recover: f32 = if ps.sanity < difficulty.0.max_recoverable_sanity() {
            4.0 / 100.0 / difficulty.0.sanity_drain_rate()
        } else {
            0.0
        };

        let miasma_drain = if let Some(miasma) = miasma.as_ref() {
            let p_val = miasma.pressure_field.get(p).copied().unwrap_or(0.0);
            // Linear addition to crazyness starting at 1000 and reaching full intensity at 10000.
            // Full intensity is roughly 2.0 extra crazyness units per second (at default rate).
            ((p_val - 1000.0) / 4500.0).max(0.0)
        } else {
            0.0
        };

        ps.crazyness +=
            (crazy.clamp(0.000000001, 10000000.0).sqrt() * 0.2 * difficulty.0.sanity_drain_rate()
                - sanity_recover * ps.crazyness / (1.0 + ps.mean_sound * 10.0)
                + miasma_drain * difficulty.0.sanity_drain_rate())
                * dt;
        if ps.crazyness < 0.0 {
            ps.crazyness = 0.0;
        }
        ps.sanity = calculate_sanity(ps.crazyness);
    }
}
