use std::mem::swap;

use bevy::prelude::*;
use rand::Rng;

use crate::{board::Position, ghost_definitions::Evidence};

use super::{on_off, playergear::EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};

#[derive(Component, Debug, Clone, Default)]
pub struct Recorder {
    pub enabled: bool,
    pub frame_counter: u32,
    pub display_secs_since_last_update: f32,
    pub sound: f32,
    pub sound_l: Vec<f32>,
    pub amt_recorded: f32,
    pub evp_recorded_time_secs: f32,
    pub evp_recorded_display: bool,
}

impl GearUsable for Recorder {
    fn get_sprite_idx(&self) -> GearSpriteID {
        if !self.enabled {
            return GearSpriteID::RecorderOff;
        }
        let mut rng = rand::thread_rng();
        let f = rng.gen_range(0.5..2.0);
        let s = self.sound * f;
        if s < 5.0 {
            return GearSpriteID::Recorder1;
        }
        if s < 15.0 {
            return GearSpriteID::Recorder2;
        }
        if s < 45.0 {
            return GearSpriteID::Recorder3;
        }
        GearSpriteID::Recorder4
    }

    fn get_display_name(&self) -> &'static str {
        "Recorder"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            if self.evp_recorded_display {
                "- EVP RECORDED -".to_string()
            } else {
                format!("Volume: {:>4.0}dB", self.sound)
            }
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
        let bpos = pos.to_board_position();
        let Some(sound) = gs.bf.sound_field.get(&bpos) else {
            return;
        };
        let sound_reading = sound.iter().sum::<Vec2>().length() * 1000.0;
        if self.sound_l.len() < 120 {
            self.sound_l.push(sound_reading);
        }
        // Double noise reduction to remove any noise from measurement.
        let n = self.frame_counter as usize % self.sound_l.len();
        self.sound_l[n] = sound_reading;
        if self.sound > 1.0 && self.enabled && gs.bf.evidences.contains(&Evidence::EVPRecording) {
            self.amt_recorded += self.sound * gs.time.delta_seconds();
            if self.amt_recorded > 200.0 {
                self.evp_recorded_time_secs = gs.time.elapsed_seconds();
                self.amt_recorded = 0.0;
            }
        }

        if self.display_secs_since_last_update > 0.1 {
            self.display_secs_since_last_update = 0.0;
            let sum_snd: f32 = self.sound_l.iter().sum();
            let avg_snd: f32 = sum_snd / self.sound_l.len() as f32 + 1.0;
            self.sound = (avg_snd.ln() * 10.0).clamp(0.0, 60.0);
            if gs.bf.evidences.contains(&Evidence::EVPRecording) {
                self.sound_l.iter_mut().for_each(|x| *x /= 1.2);
                self.evp_recorded_display =
                    (gs.time.elapsed_seconds() - self.evp_recorded_time_secs) < 2.0;
            } else {
                self.sound_l.iter_mut().for_each(|x| *x /= 2.0);
            }
            if self.sound > 1.0 && self.enabled {
                gs.play_audio("sounds/effects-radio-scan.ogg".into(), self.sound / 60.0);
            }
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl From<Recorder> for Gear {
    fn from(value: Recorder) -> Self {
        Gear::new_from_kind(GearKind::Recorder(value))
    }
}

pub fn sound_update(
    mut bf: ResMut<crate::board::BoardData>,
    roomdb: Res<crate::board::RoomDB>,
    qg: Query<(&crate::ghost::GhostSprite, &Position)>,
) {
    let mut rng = rand::thread_rng();
    let gn = rng.gen_range(0..30_u32);

    if gn == 0 {
        // Ghost talk once in a while
        for (_, pos) in qg.iter() {
            let bpos = pos.to_board_position();

            for _ in 0..16 {
                let mut v = Vec2::new(rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0));
                let l = v.length();
                if l < 0.02 {
                    continue;
                }
                let loudness = rng.gen_range(0.3_f32..2.0).powi(2);
                v *= loudness * 4.5 / l;
                let vn = v.normalize() * 1.5;
                let newbpos = Position {
                    x: (bpos.x as f32 + vn.x),
                    y: (bpos.y as f32 + vn.y),
                    z: bpos.z as f32,
                    global_z: 0.0,
                }
                .to_board_position();
                bf.sound_field.entry(newbpos).or_default().push(v);
            }
        }
    }

    let map_pos = bf.sound_field.keys().cloned().collect::<Vec<_>>();
    for mpos in map_pos.into_iter() {
        let Some(s_v) = bf.sound_field.get_mut(&mpos) else {
            continue;
        };
        let mut data: Vec<Vec2> = vec![];
        swap(s_v, &mut data);
        if data.is_empty() {
            continue;
        }
        let v: Vec2 = data.into_iter().sum();
        let sz = (v.length() * 2.0).ceil() as usize;
        for _ in 0..sz {
            let mut v = v;
            v.x += rng.gen_range(-0.05..0.05) + rng.gen_range(-0.05..0.05);
            v.y += rng.gen_range(-0.05..0.05) + rng.gen_range(-0.05..0.05);
            v /= 1.05 * sz as f32;
            let mut v1 = v.normalize() * 1.1;
            v1 *= rng.gen_range(0.1..1.0);
            v1.x += rng.gen_range(-0.3..0.3) + rng.gen_range(-0.3..0.3);
            v1.y += rng.gen_range(-0.3..0.3) + rng.gen_range(-0.3..0.3);
            let n_p = Position {
                x: mpos.x as f32 + v1.x,
                y: mpos.y as f32 + v1.y,
                z: mpos.z as f32,
                global_z: 0.0,
            };
            let bn_p = n_p.to_board_position();

            if roomdb.room_tiles.get(&bn_p).is_some() && v.length() > 0.00002 {
                bf.sound_field.entry(bn_p).or_default().push(v);
            }
        }
    }
}
