use crate::metrics;

use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng;
use std::mem::swap;
use uncore::random_seed;
use uncore::{
    components::{board::position::Position, ghost_sprite::GhostSprite},
    metric_recorder::SendMetric,
    resources::{board_data::BoardData, roomdb::RoomDB},
    types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition},
}; 

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
    pub evp_recorded_count: usize,
    pub display_glitch_timer: f32, // Added for EMI effects
    pub false_reading_timer: f32,  // For creating false audio spikes
    pub blinking_hint_active: bool,
}

impl GearUsable for Recorder {
    fn get_sprite_idx(&self) -> GearSpriteID {
        if !self.enabled {
            return GearSpriteID::RecorderOff;
        }

        // Randomly glitch the display during interference
        if self.display_glitch_timer > 0.0 && random_seed::rng().random_range(0.0..1.0) < 0.4 {
            return match random_seed::rng().random_range(0..3) {
                0 => GearSpriteID::RecorderOff,
                1 => GearSpriteID::Recorder4, // Show max reading
                _ => GearSpriteID::Recorder1,
            };
        }

        // Normal operation
        let mut rng = random_seed::rng();
        let f = rng.random_range(0.5..2.0);

        // Add artificial noise during false readings
        let mut s = self.sound;
        if self.false_reading_timer > 0.0 {
            s = s * 2.5 + 30.0 * self.false_reading_timer;
        }
        s *= f;

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

    fn get_description(&self) -> &'static str {
        "Records ambient sounds and conversations. Sometimes it can capture EVP phenomena."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);

        // Show garbled text when glitching
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..5) {
                0 => "Vol: ****ERROR****",
                1 => "INTERFERENCE DETE---",
                2 => "--EVP D??E*T?D--",
                3 => "SIGNAL:NOISE=0.---",
                _ => "AUDIO MALFUNCTION",
            };
            return format!("{name}: {on_s}\n{garbled}");
        }

        // Normal display
        let msg = if self.enabled {
            if self.evp_recorded_display {
                // This state implies evidence has been found and is being actively displayed
                if self.blinking_hint_active {
                    if self.frame_counter % 40 < 20 {
                        "- EVP RECORDED !!! -".to_string()
                    } else {
                        "- EVP RECORDED     -".to_string()
                    }
                } else {
                    "- EVP RECORDED -".to_string()
                }
            } else {
                format!(
                    "Volume: {:>4.0}dB ({})",
                    self.sound, self.evp_recorded_count
                )
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

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let mut rng = random_seed::rng();
        self.display_secs_since_last_update += gs.time.delta_secs();
        self.frame_counter += 1;
        self.frame_counter %= 65413;

        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();

            // Play static/interference sounds when glitching
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.4 {
                // More static sounds than other devices - this is an audio device after all
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.5, pos);
            }
        }

        // Decrement false reading timer if active
        if self.false_reading_timer > 0.0 {
            self.false_reading_timer -= gs.time.delta_secs();

            // Play EVP-like sounds during false readings
            if self.enabled && random_seed::rng().random_range(0.0..1.0) < 0.3 {
                let volume = 0.3 + self.false_reading_timer * 0.3;
                gs.play_audio("sounds/effects-radio-scan.ogg".into(), volume, pos);
            }
        }

        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }

        // Regular recorder functionality
        const K: f32 = 0.5;
        let pos = Position {
            x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
            y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
            z: pos.z,
            global_z: pos.global_z,
        };

        // Rest of original update code...
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
        if self.enabled {
            self.sound_l[n] = sound_reading;
        } else {
            self.sound_l[n] = 0.0;
            self.evp_recorded_count = 0;
        }
        if self.sound > 1.0 && self.enabled && gs.bf.evidences.contains(&Evidence::EVPRecording) {
            self.amt_recorded += self.sound * gs.time.delta_secs();
            if self.amt_recorded > 200.0 {
                self.evp_recorded_time_secs = gs.time.elapsed_secs();
                self.evp_recorded_count += 1;
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
                    (gs.time.elapsed_secs() - self.evp_recorded_time_secs) < 2.0;
            } else {
                self.sound_l.iter_mut().for_each(|x| *x /= 2.0);
            }
            if self.sound > 1.0 && self.enabled {
                gs.play_audio(
                    "sounds/effects-radio-scan.ogg".into(),
                    self.sound / 60.0,
                    &pos,
                );
            }
        }

        // Update blinking_hint_active
        const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
        // We consider "showing evidence strongly" when an EVP has been recorded at least once.
        // The `evp_recorded_display` flag is for the temporary "- EVP RECORDED -" message,
        // while `evp_recorded_count > 0` indicates that evidence has been found.
        if self.evp_recorded_count > 0 && self.display_glitch_timer <= 0.0 {
            let count = gs
                .player_profile
                .times_evidence_acknowledged_on_gear
                .get(&Evidence::EVPRecording)
                .copied()
                .unwrap_or(0);
            self.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
        } else {
            self.blinking_hint_active = false;
        }
    }

    fn is_status_text_showing_evidence(&self) -> f32 {
        if self.is_enabled() && self.display_glitch_timer <= 0.0 && self.evp_recorded_count > 0 {
            1.0
        } else {
            0.0
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn is_blinking_hint_active(&self) -> bool {
        self.blinking_hint_active
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
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) * 0.7 {
            self.display_glitch_timer = rng.random_range(0.2..0.5);
        }

        // Effect 2: False EVP/high volume readings
        // More likely than display glitches - this is audio equipment after all
        if rng.random_range(0.0..1.0) < effect_strength.powi(2) * 1.2 {
            self.false_reading_timer = rng.random_range(0.3..0.8);
        }
    }
}

impl From<Recorder> for Gear {
    fn from(value: Recorder) -> Self {
        Gear::new_from_kind(GearKind::Recorder, value.box_clone())
    }
}

pub fn sound_update(
    mut bf: ResMut<BoardData>,
    roomdb: Res<RoomDB>,
    qg: Query<(&GhostSprite, &Position)>,
) {
    let measure = metrics::SOUND_UPDATE.time_measure();

    let mut rng = random_seed::rng();
    let gn = rng.random_range(0..30_u32);
    if gn == 0 {
        // Ghost talk once in a while
        for (_, pos) in qg.iter() {
            let bpos = pos.to_board_position();
            for _ in 0..16 {
                let mut v = Vec2::new(rng.random_range(-2.0..2.0), rng.random_range(-2.0..2.0));
                let l = v.length();
                if l < 0.02 {
                    continue;
                }
                let loudness = rng.random_range(0.3_f32..2.0).powi(2);
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
            v.x += rng.random_range(-0.05..0.05) + rng.random_range(-0.05..0.05);
            v.y += rng.random_range(-0.05..0.05) + rng.random_range(-0.05..0.05);
            v /= 1.05 * sz as f32;
            let mut v1 = v.normalize() * 1.1;
            v1 *= rng.random_range(0.1..1.0);
            v1.x += rng.random_range(-0.3..0.3) + rng.random_range(-0.3..0.3);
            v1.y += rng.random_range(-0.3..0.3) + rng.random_range(-0.3..0.3);
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

    measure.end_ms();
}
