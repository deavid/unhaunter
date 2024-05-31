use bevy::prelude::*;
use rand::Rng as _;

use crate::{board::Position, ghost_definitions::Evidence};

use super::{on_off, playergear::EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct GeigerCounter {
    pub enabled: bool,
    pub display_secs_since_last_update: f32,
    pub frame_counter: u16,
    pub sound_a1: f32,
    pub sound_a2: f32,
    pub sound_display: f32,
    pub sound_l: Vec<f32>,
    pub last_sound_time_secs: f32,
}

impl GearUsable for GeigerCounter {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => {
                if self.sound_a1 > 10.0 {
                    GearSpriteID::GeigerTick
                } else {
                    GearSpriteID::GeigerOn
                }
            }
            false => GearSpriteID::GeigerOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Geiger Counter"
    }
    fn get_description(&self) -> &'static str {
        "Measures radioactivity by counting alpha and beta particles. It can be used to roughly locate the ghost with patience."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            format!("Reading: {:.1}cpm", self.sound_display)
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        self.enabled = !self.enabled;
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let mut rng = rand::thread_rng();
        self.display_secs_since_last_update += gs.time.delta_seconds();
        self.frame_counter += 1;
        self.frame_counter %= 65413;
        const K: f32 = 0.5;
        let pos = Position {
            x: pos.x + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            y: pos.y + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            z: pos.z + rng.gen_range(-K..K) + rng.gen_range(-K..K),
            global_z: pos.global_z,
        };
        let dist2breach = gs.bf.breach_pos.distance2(&pos) + 10.0;
        let breach_energy = dist2breach.recip() * 20000.0;
        let bpos = pos.to_board_position();
        for (i, bpos) in bpos.xy_neighbors(4).iter().enumerate() {
            let sound = gs.bf.sound_field.get(bpos).cloned().unwrap_or_default();
            let sound_reading = sound.iter().sum::<Vec2>().length() * 1000.0;
            if self.sound_l.len() < 1200 {
                self.sound_l.push(sound_reading);
            }
            let n = (self.frame_counter as usize + i) % self.sound_l.len();
            self.sound_l[n] /= 4.0;
            if self.enabled {
                self.sound_l[n] += sound_reading * 40.0 + breach_energy;
            }
        }
        let sum_snd: f32 = self.sound_l.iter().sum();
        let avg_snd: f32 = sum_snd / self.sound_l.len() as f32;
        let avg_snd = if gs.bf.evidences.contains(&Evidence::CPM500) {
            f32::tanh(avg_snd.sqrt() / 20.0) * 980.0
        } else {
            f32::tanh(avg_snd.sqrt() / 10.0) * 480.0
        };
        const MASS: f32 = 16.0;
        if self.enabled {
            self.sound_a1 = (self.sound_a1 * MASS + avg_snd * MASS.recip()) / (MASS + MASS.recip());
            self.sound_a2 =
                (self.sound_a2 * MASS + self.sound_a1 * MASS.recip()) / (MASS + MASS.recip());
        } else {
            self.sound_a1 /= 1.01;
            self.sound_a2 /= 1.01;
        }
        self.sound_l.iter_mut().for_each(|x| *x /= 1.06);

        if gs.time.elapsed_seconds() - self.last_sound_time_secs > 60.0 / avg_snd && self.enabled {
            self.last_sound_time_secs = gs.time.elapsed_seconds() + rng.gen_range(0.01..0.02);
            gs.play_audio("sounds/effects-chirp-click.ogg".into(), 0.25);
        }

        if self.display_secs_since_last_update > 0.5 {
            self.display_secs_since_last_update = 0.0;
            self.sound_display = self.sound_a2;
        }
    }
    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<GeigerCounter> for Gear {
    fn from(value: GeigerCounter) -> Self {
        Gear::new_from_kind(GearKind::GeigerCounter(value))
    }
}
