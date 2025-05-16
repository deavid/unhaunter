use uncore::random_seed;
use uncore::systemparam::gear_stuff::GearStuff;

use uncore::{
    components::board::position::Position,
    types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition},
};

use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng as _;

#[derive(Debug, Clone, Default)]
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
    pub enabled: bool,
    pub frame_counter: u16,
    pub temp_l2: Vec<f32>,
    pub temp_l1: f32,
    pub emf: f32,
    pub emf_level: EMFLevel,
    pub miasma_pressure: f32,
    pub miasma_pressure_2: f32,
    pub last_sound_secs: f32,
    pub last_meter_update_secs: f32,
    pub display_glitch_timer: f32,
}

impl GearUsable for EMFMeter {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => {
                // Occasionally show blank screen during glitch
                if self.display_glitch_timer > 0.0
                    && random_seed::rng().random_range(0.0..1.0) < 0.3
                {
                    // Alternative if EMFMeterGlitch doesn't exist yet
                    if self.display_glitch_timer > 0.0
                        && random_seed::rng().random_range(0.0..1.0) < 0.3
                    {
                        // Temporarily show as off or randomly flicker between levels
                        match random_seed::rng().random_range(0..3) {
                            0 => GearSpriteID::EMFMeterOff,
                            1 => GearSpriteID::EMFMeter4,
                            _ => self.emf_level.to_spriteid(),
                        }
                    } else {
                        self.emf_level.to_spriteid()
                    }
                } else {
                    self.emf_level.to_spriteid()
                }
            }
            false => GearSpriteID::EMFMeterOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "EMF Meter"
    }

    fn get_description(&self) -> &'static str {
        "Used to find electric wires behind walls. Ghosts might disturb the electromagnetic field."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);

        // Show garbled text when glitching
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Reading: ERR0R\nEnergy: ###.###",
                1 => "Reading: ---.--\nEnergy: FAULT",
                2 => "INTERFERENCE DET---\nCALIBRATING...",
                _ => "Signal Lost\nReacquiring...",
            };
            return format!("{name}:  {on_s}\n{garbled}");
        }

        // Regular display
        let msg = if self.enabled {
            format!(
                "Reading: {:>6.1}mG {}\nEnergy: {:>9.3}T",
                self.emf,
                self.emf_level.to_status(),
                self.miasma_pressure_2,
            )
        } else {
            "".to_string()
        };
        format!("{name}:  {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        self.enabled = !self.enabled;
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, ep: &EquipmentPosition) {
        let mut rng = random_seed::rng();
        self.frame_counter += 1;
        if self.frame_counter > 65413 {
            self.frame_counter = 0;
        }
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

            self.miasma_pressure = self.miasma_pressure * F + miasma_pressure * (1.0 - F);
        }
        self.miasma_pressure_2 = self.miasma_pressure_2 * F + self.miasma_pressure * (1.0 - F);

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
        if self.temp_l2.len() < 2 {
            self.temp_l2.push(temp_reading);
        }

        // Double noise reduction to remove any noise from measurement.
        let n = self.frame_counter as usize % self.temp_l2.len();
        self.temp_l2[n] = (self.temp_l2[n] * air_mass + temp_reading) / (air_mass + 1.0);
        self.temp_l1 = (self.temp_l1 * air_mass + temp_reading) / (air_mass + 1.0);
        if self.temp_l2.len() < 40 {
            self.temp_l2.push(self.temp_l1);
        }
        let sec = gs.time.elapsed_secs();
        if self.last_meter_update_secs + 0.5 < sec {
            self.last_meter_update_secs = sec;
            let sum_temp: f32 = self.temp_l2.iter().sum();
            let avg_temp: f32 = sum_temp / self.temp_l2.len() as f32;
            let mut new_emf = (avg_temp - self.temp_l1).abs() * 3.0;
            self.emf -= 0.2 * gs.difficulty.0.equipment_sensitivity;
            self.emf /= 1.4_f32.powf(gs.difficulty.0.equipment_sensitivity);
            if gs.bf.evidences.contains(&Evidence::EMFLevel5) {
                new_emf = f32::tanh(new_emf / 40.0) * 45.0;
            } else {
                // If there's no possibility of EMF5, remap it so it's always below 20 mGauss.
                new_emf = f32::tanh(new_emf / 19.0) * 15.0;
            }
            self.emf = self.emf.max(new_emf);
            self.emf_level = EMFLevel::from_milligauss(self.emf);
        }
        if self.enabled {
            let delta = 10.0 / (self.emf + 0.5).powf(1.5);
            if self.last_sound_secs + delta < sec {
                self.last_sound_secs = sec;
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

        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();

            // Play static/interference sound when glitching
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.5 {
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.4, pos);
            }
        }

        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }
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

        // Random EMF spikes
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
            self.emf = (rng.random_range(0.0..1.0) * effect_strength * 5.0).min(11.0);
            self.emf_level = EMFLevel::from_milligauss(self.emf);
            // Jumble numbers temporarily
            self.display_glitch_timer = 0.2;
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn can_enable(&self) -> bool {
        true
    }
}

impl From<EMFMeter> for Gear {
    fn from(value: EMFMeter) -> Self {
        Gear::new_from_kind(GearKind::EMFMeter, value.box_clone())
    }
}
