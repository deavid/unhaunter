use uncore::random_seed;
use uncore::systemparam::gear_stuff::GearStuff;
use uncore::{
    components::board::position::Position,
    types::{evidence::Evidence, gear::equipmentposition::EquipmentPosition},
};

use super::{Gear, GearKind, GearSpriteID, GearUsable, on_off};
use bevy::prelude::*;
use rand::Rng as _;

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct GeigerCounter {
    pub enabled: bool,
    pub display_secs_since_last_update: f32,
    pub frame_counter: u16,
    pub sound_a1: f32,
    pub sound_a2: f32,
    pub sound_display: f32, // Used for the display value
    pub sound_l: Vec<f32>,
    pub last_sound_time_secs: f32,
    pub display_glitch_timer: f32,
    pub output_sound: f32,
}

impl GeigerCounter {
    pub fn calculate_output_sound(&self, gs: &GearStuff) -> f32 {
        // if the glitch timer is running, then apply a random value.
        if self.display_glitch_timer > 0.0 {
            let mut rng = random_seed::rng();
            return rng.random_range(0.0..1000.0);
        }
        let sum_snd: f32 = self.sound_l.iter().sum();
        let avg_snd: f32 = sum_snd / self.sound_l.len() as f32;
        if gs.bf.evidences.contains(&Evidence::CPM500) {
            f32::tanh(avg_snd.sqrt() / 20.0) * 980.0
        } else {
            f32::tanh(avg_snd.sqrt() / 10.0) * 480.0
        }
    }
}

impl GearUsable for GeigerCounter {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => {
                if self.display_glitch_timer > 0.0 {
                    if random_seed::rng().random_bool(0.7) {
                        GearSpriteID::GeigerOff
                    } else {
                        GearSpriteID::GeigerTick
                    }
                } else if self.sound_a1 > 10.0 {
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
        if self.enabled && self.display_glitch_timer > 0.0 {
            let garbled = match random_seed::rng().random_range(0..4) {
                0 => "Reading: ERR0R\nEnergy: ###.###",
                1 => "Reading: ---.--\nEnergy: FAULT",
                2 => "INTERFERENCE DET---\nCALIBRATING...",
                _ => "Signal Lost\nReacquiring...",
            };
            return format!("{name}:  {on_s}\n{garbled}");
        }
        let msg = if self.enabled {
            // Use sound_display here, NOT output_sound!
            format!("Reading: {:.1}cpm", self.sound_display)
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut GearStuff) {
        self.enabled = !self.enabled;
    }

    fn update(&mut self, gs: &mut GearStuff, pos: &Position, _ep: &EquipmentPosition) {
        let mut rng = random_seed::rng();
        self.display_secs_since_last_update += gs.time.delta_secs(); // Increment the timer
        self.frame_counter += 1;
        self.frame_counter %= 65413;
        const K: f32 = 0.5;
        let posk = Position {
            x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
            y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
            z: pos.z + rng.random_range(-K..K) + rng.random_range(-K..K),
            global_z: pos.global_z,
        };
        let dist2breach = gs.bf.breach_pos.distance2(&posk) + 10.0;
        let breach_energy = dist2breach.recip() * 20000.0;
        let bpos = posk.to_board_position();
        for (i, bpos) in bpos.iter_xy_neighbors_nosize(4).enumerate() {
            let sound = gs.bf.sound_field.get(&bpos).cloned().unwrap_or_default();
            let sound_reading = sound.iter().sum::<Vec2>().length() * 1000.0;
            if self.sound_l.len() < 1200 {
                self.sound_l.push(sound_reading);
            }
            let n = (self.frame_counter as usize + i) % self.sound_l.len();
            self.sound_l[n] /= 4.0 * gs.difficulty.0.equipment_sensitivity;
            if self.enabled {
                self.sound_l[n] +=
                    sound_reading * 40.0 + breach_energy * gs.difficulty.0.equipment_sensitivity;
            }
        }

        self.sound_l.iter_mut().for_each(|x| *x /= 1.06);

        let mass: f32 = 12.0 / gs.difficulty.0.equipment_sensitivity;
        if self.enabled {
            // Calculate the *current* output sound.
            let current_output_sound = self.calculate_output_sound(gs);

            // Smooth the *current* output to get sound_a1 (first IIR filter).
            self.sound_a1 = (self.sound_a1 * mass + current_output_sound * mass.recip())
                / (mass + mass.recip());

            // Smooth sound_a1 to get output_sound (second IIR filter).
            // Crucial change:  Use sound_a1 *here*, not current_output_sound!
            self.output_sound = (self.output_sound * 10.0 + self.sound_a1) / 11.0;

            self.sound_a2 =
                (self.sound_a2 * mass + self.sound_a1 * mass.recip()) / (mass + mass.recip());
        } else {
            self.sound_a1 /= 1.01;
            self.sound_a2 /= 1.01;
        }

        if gs.time.elapsed_secs() - self.last_sound_time_secs > 60.0 / self.sound_a1 && self.enabled
        {
            if self.display_glitch_timer <= 0.0001 {
                self.last_sound_time_secs = gs.time.elapsed_secs() + rng.random_range(0.01..0.02);
                gs.play_audio("sounds/effects-chirp-click.ogg".into(), 0.25, pos);
            } else {
                self.last_sound_time_secs = gs.time.elapsed_secs() + rng.random_range(0.01..0.02);
                gs.play_audio("sounds/effects-chirp-short.ogg".into(), 0.25, pos);
            }
        }
        // Update sound_display *only* if enough time has passed.
        if self.display_secs_since_last_update > 0.5 {
            self.display_secs_since_last_update = 0.0; // Reset the timer
            self.sound_display = self.output_sound; // Update the display value
        }
        // Decrement glitch timer if active
        if self.display_glitch_timer > 0.0 {
            self.display_glitch_timer -= gs.time.delta_secs();
        }
        // Apply EMI if warning is active and we're electronic
        if let Some(ghost_pos) = &gs.bf.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            self.apply_electromagnetic_interference(gs.bf.ghost_warning_intensity, distance2);
        }
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
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
            self.sound_a1 = rng.random_range(0.0..400.0);
            // Jumble numbers temporarily
            self.display_glitch_timer = 0.3;
        }
    }
}

impl From<GeigerCounter> for Gear {
    fn from(value: GeigerCounter) -> Self {
        Gear::new_from_kind(GearKind::GeigerCounter, value.box_clone())
    }
}
