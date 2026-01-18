use bevy_persistent::Persistent;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfog_core::miasma::MiasmaGrid;
use unfoundation_core::random_seed;
use ungear_core::components::core::{
    Battery, Electronic, GearSprite, ItemName, PerceivedClarity, StatusText,
};
use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::Toggleable;
use unprofile_core::profile::PlayerProfileData;
use unsound_core::emitter::SoundEmitter;
use unsound_core::resources::SoundGrid;
use unthermal_core::resources::ThermalGrid;

use unghost_core::types::evidence::Evidence;
use unspatial_core::position::Position;

use bevy::prelude::*;
use rand::Rng as _;
use unfoundation_core::types::gear::EquipmentPosition;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::emfmeter::{EMFLevel, EMFMeter};
use unrender_std::resources::sprite_registry::GearSpriteID;

pub(crate) fn update_emfmeter(
    mut q_emf: Query<(
        &mut EMFMeter,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &mut Battery,
        &Electronic,
        &Position,
        &ItemName,
        &EquipmentPosition,
        &mut PerceivedClarity,
    )>,
    mut gs_audio: SoundEmitter,
    miasma: Res<MiasmaGrid>,
    tg: Res<ThermalGrid>,
    sg: Res<SoundGrid>,
    difficulty: Res<CurrentDifficulty>,
    haunt_state: Res<HauntState>,
    player_profile: Res<Persistent<PlayerProfileData>>,
) {
    for (
        mut emf,
        mut status,
        mut sprite,
        toggle,
        mut battery,
        electronic,
        pos,
        name,
        ep,
        mut perceived_clarity,
    ) in q_emf.iter_mut()
    {
        let mut rng = random_seed::rng();
        emf.frame_counter = emf.frame_counter.wrapping_add(1);

        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        // Update Sprite
        if toggle.is_on {
            if electronic.glitch_timer > 0.0 && random_seed::rng().random_range(0.0..1.0) < 0.3 {
                // Flicker when glitching but enabled
                sprite.0 = match random_seed::rng().random_range(0..3) {
                    0 => GearSpriteID::EMFMeterOff.to_visual_key(),
                    1 => GearSpriteID::EMFMeter4.to_visual_key(), // Example: flicker to a high reading or specific glitch sprite
                    _ => emf.emf_level.to_spriteid().to_visual_key(), // Or back to its current reading sprite
                };
            } else {
                // Normal operation, not glitching or glitch not causing visual disruption this frame
                sprite.0 = emf.emf_level.to_spriteid().to_visual_key();
            }
        } else {
            sprite.0 = GearSpriteID::EMFMeterOff.to_visual_key();
        }

        // Update Logic
        if toggle.is_on {
            const K: f32 = 0.5;
            const F: f32 = 0.95;
            for _ in 0..20 {
                let pos = Position {
                    x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
                    y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
                    z: pos.z,
                    visual_priority: pos.visual_priority,
                };
                let bpos = pos.to_board_position();

                let miasma_pressure = miasma.pressure_field[bpos.ndidx()];

                emf.miasma_pressure = emf.miasma_pressure * F + miasma_pressure * (1.0 - F);
            }
            emf.miasma_pressure_2 = emf.miasma_pressure_2 * F + emf.miasma_pressure * (1.0 - F);

            let posk = Position {
                x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
                y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
                z: pos.z,
                visual_priority: pos.visual_priority,
            };
            let bpos = posk.to_board_position();

            let temperature = tg.temperature_field[bpos.ndidx()];
            let sound = sg.sound_field.get(&bpos).cloned().unwrap_or_default();
            let sound_reading = sound.iter().sum::<Vec2>().length() * 100.0;
            let temp_reading = temperature / 10.0 + sound_reading;
            let air_mass: f32 = 5.0 / difficulty.0.equipment_sensitivity;
            if emf.temp_l2.len() < 2 {
                emf.temp_l2.push(temp_reading);
            }

            // Double noise reduction to remove any noise from measurement.
            let n = emf.frame_counter as usize % emf.temp_l2.len();
            emf.temp_l2[n] = (emf.temp_l2[n] * air_mass + temp_reading) / (air_mass + 1.0);
            emf.temp_l1 = (emf.temp_l1 * air_mass + temp_reading) / (air_mass + 1.0);
            if emf.temp_l2.len() < 40 {
                let temp_l1 = emf.temp_l1;
                emf.temp_l2.push(temp_l1);
            }
            let sec = gs_audio.time.elapsed_secs();
            if emf.last_meter_update_secs + 0.5 < sec {
                emf.last_meter_update_secs = sec;
                let sum_temp: f32 = emf.temp_l2.iter().sum();
                let avg_temp: f32 = sum_temp / emf.temp_l2.len() as f32;
                let mut new_emf = (avg_temp - emf.temp_l1).abs() * 3.0;
                emf.emf -= 0.2 * difficulty.0.equipment_sensitivity;
                emf.emf /= 1.4_f32.powf(difficulty.0.equipment_sensitivity);
                let emf5_evidence = haunt_state.ghost_dynamics.emf_level5_clarity.max(-0.2);
                new_emf = f32::tanh(new_emf / (20.0 + emf5_evidence * 20.0))
                    * (15.0 + emf5_evidence * 30.0);
                emf.emf = emf.emf.max(new_emf);
                emf.emf_level = EMFLevel::from_milligauss(emf.emf);

                // Update blinking_hint_active
                const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                if emf.emf_level == EMFLevel::EMF5 {
                    let count = player_profile
                        .times_evidence_acknowledged_on_gear
                        .get(&Evidence::EMFLevel5)
                        .copied()
                        .unwrap_or(0);
                    emf.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
                }
            }
            if toggle.is_on {
                let delta = 10.0 / (emf.emf + 0.5).powf(1.5);
                if emf.last_sound_secs + delta < sec {
                    emf.last_sound_secs = sec;
                    match ep {
                        EquipmentPosition::Hand(_) => {
                            gs_audio.play_audio("sounds/effects-chirp-shorter.ogg".into(), 1.0, pos)
                        }
                        EquipmentPosition::Stowed => {
                            gs_audio.play_audio("sounds/effects-chirp-shorter.ogg".into(), 0.5, pos)
                        }
                        EquipmentPosition::Deployed => {
                            gs_audio.play_audio("sounds/effects-chirp-shorter.ogg".into(), 0.7, pos)
                        }
                    }
                }
            }

            // Play static/interference sound when glitching
            if electronic.glitch_timer > 0.0
                && toggle.is_on
                && random_seed::rng().random_range(0.0..1.0) < 0.5
            {
                gs_audio.play_audio("sounds/effects-chirp-short.ogg".into(), 0.4, pos);
            }
        }

        // Update Status Text
        let on_s = on_off(toggle.is_on);

        // Show garbled text when enabled but glitching
        if toggle.is_on && electronic.glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Reading: ERR0R\nEnergy: ###.###",
                1 => "Reading: ---.--\nEnergy: FAULT",
                2 => "INTERFERENCE DET---\nCALIBRATING...",
                _ => "Signal Lost\nReacquiring...",
            };
            status.0 = format!("{}:  {}\n{}", name.0, on_s, garbled);
            continue;
        }

        // Regular display
        let msg = if toggle.is_on {
            let emf_status_text = emf.emf_level.to_status();
            let blinking_emf_text = if emf.frame_counter % 30 < 15
                && emf.blinking_hint_active
                && emf.emf_level == EMFLevel::EMF5
            {
                format!(">[{}]<", emf_status_text)
            } else {
                format!("  {}  ", emf_status_text)
            };
            format!(
                "Reading: {:>6.1}mG {}\nEnergy: {:>9.3}T",
                emf.emf, blinking_emf_text, emf.miasma_pressure_2,
            )
        } else {
            "".to_string()
        };
        status.0 = format!("{}:  {}\n{}", name.0, on_s, msg);

        perceived_clarity.from_status_text =
            if toggle.is_on && emf.emf_level == EMFLevel::EMF5 && electronic.glitch_timer <= 0.0 {
                1.0
            } else {
                0.0
            };
        perceived_clarity.from_icon = perceived_clarity.from_status_text;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_emfmeter);
}
