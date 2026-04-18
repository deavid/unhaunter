use bevy::prelude::*;
use bevy_persistent::Persistent;
use rand::RngExt;
use uncommon_app_core::random_seed;
use uncommon_app_core::utils::temperature::kelvin_to_celsius;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use ungear_core::components::core::{
    Battery, Electronic, GearSprite, ItemName, PerceivedClarity, StatusText, StatusTextRefreshTimer,
};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
use ungearitems_core::components::thermometer::Thermometer;
use uninteraction_core::interaction::Toggleable;
use uninvestigation_core::evidence::Evidence;
use unmetrics_core::metrics::SendMetric;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::position::Position;
use unthermal_core::resources::ThermalGrid;

use crate::metrics;

pub(crate) fn update_thermometer(
    mut commands: Commands,
    mut q_thermometer: Query<(
        Entity,
        &mut Thermometer,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &mut Battery,
        &Electronic,
        &Position,
        &ItemName,
        &mut PerceivedClarity,
        Has<StatusTextRefreshTimer>,
    )>,
    tg: If<Res<ThermalGrid>>,
    difficulty: Res<CurrentDifficulty>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    authority: Option<Res<unreplicon_core::resources::AuthorityRole>>,
) {
    let measure = metrics::TEMPERATURE_UPDATE.time_measure();
    let is_authority = authority.is_some();
    for (
        entity,
        mut thermometer,
        mut status,
        mut sprite,
        toggle,
        mut battery,
        electronic,
        pos,
        name,
        mut perceived_clarity,
        has_timer,
    ) in q_thermometer.iter_mut()
    {
        let mut rng = random_seed::rng();
        thermometer.frame_counter = thermometer.frame_counter.wrapping_add(1);

        // Update Battery Drain Rate
        if is_authority {
            battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };
        }

        // Update Sprite
        sprite.0 = match toggle.is_on {
            true => GearSpriteID::ThermometerOn.to_visual_key(),
            false => GearSpriteID::ThermometerOff.to_visual_key(),
        };

        // Update Logic
        if toggle.is_on {
            // We update the thermometer temperature locally on both client and server.
            // This ensures the thermometer is responsive even if the server is slow
            // to sync or if we want local simulation to take precedence.
            const K: f32 = 0.7;
            let pos = Position {
                x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
                y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
                z: pos.z,
                visual_priority: pos.visual_priority,
            };
            let bpos = pos.to_board_position();
            let temperature = bpos
                .ndidx_checked(tg.temperature_field.dim())
                .map(|idx| tg.temperature_field[idx])
                .unwrap_or(tg.ambient_temp);
            let temp_reading = temperature;
            let air_mass: f32 = 50.0 / difficulty.0.equipment_sensitivity();

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
            }

            if thermometer.frame_counter % 5 == 0 {
                // Update blinking_hint_active
                const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                if kelvin_to_celsius(thermometer.temp) < 0.0 && electronic.glitch_timer <= 0.0 {
                    let count = player_profile
                        .times_evidence_acknowledged_on_gear
                        .get(&Evidence::FreezingTemp)
                        .copied()
                        .unwrap_or(0);
                    thermometer.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
                } else {
                    thermometer.blinking_hint_active = false;
                }
            }

            // Ensure blinking_hint_active is false if not updating temp this frame,
            // or if we want it to strictly follow the evidence condition.
            // For now, let's ensure it's false if the condition isn't met.
            if !(kelvin_to_celsius(thermometer.temp) < 0.0 && electronic.glitch_timer <= 0.0) {
                thermometer.blinking_hint_active = false;
            }
        }

        // Update Status Text
        let on_s = on_off(toggle.is_on);

        // Show garbled text when glitching
        let new_status_msg = if toggle.is_on && electronic.glitch_timer > 0.0 {
            match random_seed::rng().random_range(0..4) {
                0 => "Temperature: ERR0R".to_string(),
                1 => "Temperature: ---.--°C".to_string(),
                2 => "Temperature: ?**.??°C".to_string(),
                _ => "SENSOR MALFUNCTION".to_string(),
            }
        } else {
            // Regular display
            if toggle.is_on {
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
            }
        };
        let new_status = format!("{}: {}\n{}", name.0, on_s, new_status_msg);
        status.update(entity, &mut commands, !has_timer, new_status);

        perceived_clarity.from_status_text = if toggle.is_on
            && kelvin_to_celsius(thermometer.temp) < 0.0
            && electronic.glitch_timer <= 0.0
        {
            1.0
        } else {
            0.0
        };
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_thermometer.run_if(resource_exists::<LocalPlayerRole>),
    );
}
