use uncore::random_seed;
use ungear::gear_stuff::GearStuff;

use uncore::{
    components::board::position::Position,
    types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition},
};

use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
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
    pub blinking_hint_active: bool,
}

impl GearUsable for EMFMeter {
    // Default is_enabled() is fine if it just checks self.enabled
    // fn is_enabled(&self) -> bool { self.enabled }

    // Default can_enable() is fine if it's always true
    // fn can_enable(&self) -> bool { true }

    // Override is_enabled to consider glitch state
    fn is_enabled(&self) -> bool {
        self.enabled && self.display_glitch_timer <= 0.0
    }

    // Override can_enable to consider glitch state
    fn can_enable(&self) -> bool {
        self.display_glitch_timer <= 0.0
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        // Use self.is_enabled() to check if the device is truly on and not glitching.
        // However, if we want to show a flickering or off state visually when it's
        // `enabled` but `display_glitch_timer > 0.0`, we need to check `self.enabled` directly here.
        if self.enabled {
            if self.display_glitch_timer > 0.0 && random_seed::rng().random_range(0.0..1.0) < 0.3 {
                // Flicker when glitching but enabled
                match random_seed::rng().random_range(0..3) {
                    0 => GearSpriteID::EMFMeterOff,
                    1 => GearSpriteID::EMFMeter4, // Example: flicker to a high reading or specific glitch sprite
                    _ => self.emf_level.to_spriteid(), // Or back to its current reading sprite
                }
            } else {
                // Normal operation, not glitching or glitch not causing visual disruption this frame
                self.emf_level.to_spriteid()
            }
        } else {
            GearSpriteID::EMFMeterOff
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
        let on_s = on_off(self.enabled); // Show "ON"/"OFF" based on the internal enabled state

        // Show garbled text when enabled but glitching
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Reading: ERR0R\nEnergy: ###.###",
                1 => "Reading: ---.--\nEnergy: FAULT",
                2 => "INTERFERENCE DET---\nCALIBRATING...",
                _ => "Signal Lost\nReacquiring...",
            };
            return format!("{name}:  {on_s}\n{garbled}");
        }

        // Regular display (when truly on and not glitching, checked by self.is_enabled())
        let msg = if self.is_enabled() {
            let emf_status_text = self.emf_level.to_status();
            let blinking_emf_text = if self.frame_counter % 30 < 15
                && self.blinking_hint_active
                && self.emf_level == EMFLevel::EMF5
            {
                format!(">[{}]<", emf_status_text)
            } else {
                format!("  {}  ", emf_status_text)
            };
            format!(
                "Reading: {:>6.1}mG {}\nEnergy: {:>9.3}T",
                self.emf, blinking_emf_text, self.miasma_pressure_2,
            )
        } else {
            "".to_string()
        };
        format!("{name}:  {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        if self.enabled {
            // If it's on, turn it off (regardless of glitch state, user can always turn off)
            self.enabled = false;
        } else if self.can_enable() {
            // If it's off and can be enabled (not glitching), turn it on
            self.enabled = true;
        }
        // If it's off and cannot be enabled (e.g., glitching), attempting to turn on does nothing.
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

            // Update blinking_hint_active
            const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
            if self.emf_level == EMFLevel::EMF5 {
                let count = gs
                    .player_profile
                    .times_evidence_acknowledged_on_gear
                    .get(&Evidence::EMFLevel5)
                    .copied()
                    .unwrap_or(0);
                self.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
            }
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

    fn is_status_text_showing_evidence(&self) -> f32 {
        if self.is_enabled() {
            if let EMFLevel::EMF5 = self.emf_level {
                return 1.0;
            }
        }
        0.0
    }

    fn is_icon_showing_evidence(&self) -> f32 {
        // The icon shows evidence if it's the EMF5 sprite (EMFMeter4)
        // and the device is truly enabled (not glitching).
        if self.is_enabled() {
            if let EMFLevel::EMF5 = self.emf_level {
                // Check if current sprite is indeed the EMF5 sprite.
                // get_sprite_idx() already considers glitches for visual representation.
                // However, for "evidence signal", we care about the underlying data if not glitching.
                return 1.0;
            }
        }
        0.0
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn is_blinking_hint_active(&self) -> bool {
        self.blinking_hint_active
    }
}

impl From<EMFMeter> for Gear {
    fn from(value: EMFMeter) -> Self {
        Gear::new_from_kind(GearKind::EMFMeter, value.box_clone())
    }
}
