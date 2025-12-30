use uncore_components::{Battery, Electronic, GearSprite, ItemName, StatusText, Toggleable};
use uncore_foundation::random_seed;
use ungear::gear_stuff::GearStuff;

use uncore_foundation::types::evidence::Evidence;
use unspatial_core::Position;

use super::{EquipmentPosition, GearSpriteID, on_off};
use bevy::prelude::*;
use rand::Rng as _;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EMFLevel {
    #[default]
    None,
    EMF2,
    EMF3,
    EMF4,
    EMF5,
}

impl EMFLevel {
    pub fn from_milligauss(mg: f32) -> EMFLevel {
        if mg > 20.0 {
            return EMFLevel::EMF5;
        }
        if mg > 10.0 {
            return EMFLevel::EMF4;
        }
        if mg > 2.5 {
            return EMFLevel::EMF3;
        }
        if mg > 1.5 {
            return EMFLevel::EMF2;
        }
        EMFLevel::None
    }

    pub fn to_spriteid(&self) -> GearSpriteID {
        match self {
            EMFLevel::None => GearSpriteID::EMFMeter0,
            EMFLevel::EMF2 => GearSpriteID::EMFMeter1,
            EMFLevel::EMF3 => GearSpriteID::EMFMeter2,
            EMFLevel::EMF4 => GearSpriteID::EMFMeter3,
            EMFLevel::EMF5 => GearSpriteID::EMFMeter4,
        }
    }

    pub fn to_status(&self) -> &'static str {
        match self {
            EMFLevel::None => "",
            EMFLevel::EMF2 => "EMF2",
            EMFLevel::EMF3 => "EMF3",
            EMFLevel::EMF4 => "EMF4",
            EMFLevel::EMF5 => "EMF5",
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct EMFMeter {
    pub frame_counter: u16,
    pub temp_l2: Vec<f32>,
    pub temp_l1: f32,
    pub emf: f32,
    pub emf_level: EMFLevel,
    pub miasma_pressure: f32,
    pub miasma_pressure_2: f32,
    pub last_sound_secs: f32,
    pub last_meter_update_secs: f32,
    pub blinking_hint_active: bool,
}

pub fn update_emfmeter(
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
    )>,
    mut gs: GearStuff,
) {
    for (mut emf, mut status, mut sprite, toggle, mut battery, electronic, pos, name, ep) in
        q_emf.iter_mut()
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
                    0 => GearSpriteID::EMFMeterOff,
                    1 => GearSpriteID::EMFMeter4, // Example: flicker to a high reading or specific glitch sprite
                    _ => emf.emf_level.to_spriteid(), // Or back to its current reading sprite
                };
            } else {
                // Normal operation, not glitching or glitch not causing visual disruption this frame
                sprite.0 = emf.emf_level.to_spriteid();
            }
        } else {
            sprite.0 = GearSpriteID::EMFMeterOff;
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
                    global_z: pos.global_z,
                };
                let bpos = pos.to_board_position();

                let miasma_pressure = gs.bf.miasma.pressure_field[bpos.ndidx()];

                emf.miasma_pressure = emf.miasma_pressure * F + miasma_pressure * (1.0 - F);
            }
            emf.miasma_pressure_2 = emf.miasma_pressure_2 * F + emf.miasma_pressure * (1.0 - F);

            let posk = Position {
                x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
                y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
                z: pos.z,
                global_z: pos.global_z,
            };
            let bpos = posk.to_board_position();

            let temperature = gs.bf.temperature_field[bpos.ndidx()];
            let sound = gs.bf.sound_field.get(&bpos).cloned().unwrap_or_default();
            let sound_reading = sound.iter().sum::<Vec2>().length() * 100.0;
            let temp_reading = temperature / 10.0 + sound_reading;
            let air_mass: f32 = 5.0 / gs.difficulty.0.equipment_sensitivity;
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
            let sec = gs.time.elapsed_secs();
            if emf.last_meter_update_secs + 0.5 < sec {
                emf.last_meter_update_secs = sec;
                let sum_temp: f32 = emf.temp_l2.iter().sum();
                let avg_temp: f32 = sum_temp / emf.temp_l2.len() as f32;
                let mut new_emf = (avg_temp - emf.temp_l1).abs() * 3.0;
                emf.emf -= 0.2 * gs.difficulty.0.equipment_sensitivity;
                emf.emf /= 1.4_f32.powf(gs.difficulty.0.equipment_sensitivity);
                let emf5_evidence = gs.haunt_state.ghost_dynamics.emf_level5_clarity.max(-0.2);
                new_emf = f32::tanh(new_emf / (20.0 + emf5_evidence * 20.0))
                    * (15.0 + emf5_evidence * 30.0);
                emf.emf = emf.emf.max(new_emf);
                emf.emf_level = EMFLevel::from_milligauss(emf.emf);

                // Update blinking_hint_active
                const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                if emf.emf_level == EMFLevel::EMF5 {
                    let count = gs
                        .player_profile
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
                            gs.play_audio("sounds/effects-chirp-shorter.ogg".into(), 1.0, pos)
                        }
                        EquipmentPosition::Stowed => {
                            gs.play_audio("sounds/effects-chirp-shorter.ogg".into(), 0.5, pos)
                        }
                        EquipmentPosition::Deployed => {
                            gs.play_audio("sounds/effects-chirp-shorter.ogg".into(), 0.7, pos)
                        }
                    }
                }
            }

            // Play static/interference sound when glitching
            if electronic.glitch_timer > 0.0
                && toggle.is_on
                && random_seed::rng().random_range(0.0..1.0) < 0.5
            {
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.4, pos);
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
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, update_emfmeter);
}
