use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use uncore::random_seed;
use uncore::{
    components::board::position::Position,
    types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition},
};

#[derive(Component, Debug, Clone, Default)]
pub struct SpiritBox {
    pub enabled: bool,
    pub mode_frame: u32,
    pub ghost_answer: bool,
    pub last_change_secs: f32,
    pub charge: f32,
    pub display_glitch_timer: f32, // New: Glitch timer
    pub false_answer_timer: f32,   // New: Timer for false answers
}

impl GearUsable for SpiritBox {
    fn get_sprite_idx(&self) -> GearSpriteID {
        // Glitch effect
        if self.display_glitch_timer > 0.0 {
            return match random_seed::rng().random_range(0..5) {
                0 => GearSpriteID::SpiritBoxOff,   // Blank/off
                1 => GearSpriteID::SpiritBoxScan1, // Flickering
                2 => GearSpriteID::SpiritBoxScan2,
                3 => GearSpriteID::SpiritBoxScan3,
                _ => GearSpriteID::SpiritBoxAns1, // Maybe show as if it answered
            };
        }

        // Normal operation
        match self.enabled {
            true => match self.ghost_answer {
                true => match self.mode_frame % 2 {
                    0 => GearSpriteID::SpiritBoxAns1,
                    _ => GearSpriteID::SpiritBoxAns2,
                },
                false => match self.mode_frame % 3 {
                    0 => GearSpriteID::SpiritBoxScan1,
                    1 => GearSpriteID::SpiritBoxScan2,
                    _ => GearSpriteID::SpiritBoxScan3,
                },
            },
            false => GearSpriteID::SpiritBoxOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Spirit Box"
    }

    fn get_description(&self) -> &'static str {
        "A modified AM Radio that constantly changes radio stations. It is said that the ghost can manipulate this to send messages to the living."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);

        // Glitch text
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..5) {
                0 => "Signal: --LOST--",
                1 => "Static....",
                2 => "....?--?---",
                3 => "MESSAG? IMPOSSI-",
                _ => "CHAOTIC SIGNALS",
            };
            return format!("{name}: {on_s}\n{garbled}");
        }

        // False answers
        if self.false_answer_timer > 0.0 {
            return format!("{name}: {on_s}\nEVP? (Static.)");
        }

        // Normal status
        let msg = if self.enabled {
            if self.ghost_answer {
                "EVP Detected!".to_string()
            } else {
                "Scanning..".to_string()
            }
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        // Don't allow toggling if currently glitching severely
        if self.display_glitch_timer <= 0.2 {
            self.enabled = !self.enabled;
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let sec = gs.time.elapsed_secs();
        let delta = sec - self.last_change_secs;
        self.mode_frame = (sec * 4.0).round() as u32;

        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();

            // Play more static sounds when glitching
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.6 {
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.5, pos);
            }
        }

        // Decrement false answer timer if active
        if self.false_answer_timer > 0.0 {
            self.false_answer_timer -= gs.time.delta_secs();

            // Play static/interference sounds during false answers
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.5 {
                gs.play_audio("sounds/effects-radio-scan.ogg".into(), 0.4, pos);
            }
        }

        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }

        if !self.enabled {
            return;
        }
        let mut rng = random_seed::rng();
        const K: f32 = 0.5;
        let posk = Position {
            x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
            y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
            z: pos.z + rng.random_range(-K..K) + rng.random_range(-K..K),
            global_z: pos.global_z,
        };
        let bpos = posk.to_board_position();
        let sound = gs.bf.sound_field.get(&bpos).cloned().unwrap_or_default();
        let sound_reading = sound.iter().sum::<Vec2>().length() * 100.0;
        if gs.bf.evidences.contains(&Evidence::SpiritBox) {
            self.charge += sound_reading;
        }
        if self.ghost_answer {
            if delta > 3.0 {
                self.ghost_answer = false;
            }
        } else if delta > 0.3 && self.false_answer_timer <= 0.0 && self.display_glitch_timer <= 0.0
        {
            self.last_change_secs = sec;
            gs.play_audio("sounds/effects-radio-scan.ogg".into(), 0.3, pos);
            let r = if self.charge > 30.0 {
                self.charge = 0.0;
                rng.random_range(0..10)
            } else {
                99
            };
            self.ghost_answer = true;
            match r {
                0 => gs.play_audio("sounds/effects-radio-answer1.ogg".into(), 0.7, pos),
                1 => gs.play_audio("sounds/effects-radio-answer2.ogg".into(), 0.7, pos),
                2 => gs.play_audio("sounds/effects-radio-answer3.ogg".into(), 0.7, pos),
                3 => gs.play_audio("sounds/effects-radio-answer4.ogg".into(), 0.4, pos),
                _ => self.ghost_answer = false,
            }
        } else if delta > 0.3 && self.false_answer_timer > 0.0 && self.display_glitch_timer <= 0.0 {
            self.last_change_secs = sec;
            self.ghost_answer = true;
            gs.play_audio("sounds/effects-radio-scan.ogg".into(), 0.4, pos);
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

        // Effect 1: Display glitches
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) * 0.8 {
            self.display_glitch_timer = rng.random_range(0.2..0.5);
        }

        // Effect 2: False answers
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) * 0.6 {
            self.false_answer_timer = rng.random_range(0.3..0.8);
            self.ghost_answer = true;
        }
    }
}

impl From<SpiritBox> for Gear {
    fn from(value: SpiritBox) -> Self {
        Gear::new_from_kind(GearKind::SpiritBox, value.box_clone())
    }
}
