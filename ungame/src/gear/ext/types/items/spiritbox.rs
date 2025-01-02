use super::{on_off, Gear, GearKind, GearSpriteID, GearUsable};
use crate::board::Position;
use bevy::prelude::*;
use rand::Rng;
use uncore::types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition};

#[derive(Component, Debug, Clone, Default)]
pub struct SpiritBox {
    pub enabled: bool,
    pub mode_frame: u32,
    pub ghost_answer: bool,
    pub last_change_secs: f32,
    pub charge: f32,
}

impl GearUsable for SpiritBox {
    fn get_sprite_idx(&self) -> GearSpriteID {
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
        let msg = if self.enabled {
            "Scanning..".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let sec = gs.time.elapsed_secs();
        let delta = sec - self.last_change_secs;
        self.mode_frame = (sec * 4.0).round() as u32;
        if !self.enabled {
            return;
        }
        let mut rng = rand::thread_rng();
        const K: f32 = 0.5;
        let pos = Position {
            x: pos.x + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            y: pos.y + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            z: pos.z + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            global_z: pos.global_z,
        };
        let bpos = pos.to_board_position();
        let sound = gs.bf.sound_field.get(&bpos).cloned().unwrap_or_default();
        let sound_reading = sound.iter().sum::<Vec2>().length() * 100.0;
        if gs.bf.evidences.contains(&Evidence::SpiritBox) {
            self.charge += sound_reading;
        }
        if self.ghost_answer {
            if delta > 3.0 {
                self.ghost_answer = false;
            }
        } else if delta > 0.3 {
            self.last_change_secs = sec;
            gs.play_audio("sounds/effects-radio-scan.ogg".into(), 0.3, &pos);
            let r = if self.charge > 30.0 {
                self.charge = 0.0;
                rng.gen_range(0..10)
            } else {
                99
            };
            self.ghost_answer = true;
            match r {
                0 => gs.play_audio("sounds/effects-radio-answer1.ogg".into(), 0.7, &pos),
                1 => gs.play_audio("sounds/effects-radio-answer2.ogg".into(), 0.7, &pos),
                2 => gs.play_audio("sounds/effects-radio-answer3.ogg".into(), 0.7, &pos),
                3 => gs.play_audio("sounds/effects-radio-answer4.ogg".into(), 0.4, &pos),
                _ => self.ghost_answer = false,
            }
        }
    }
}

impl From<SpiritBox> for Gear {
    fn from(value: SpiritBox) -> Self {
        Gear::new_from_kind(GearKind::SpiritBox(value.box_clone()))
    }
}
