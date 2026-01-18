use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::random_seed;
use ungear_core::gear_stuff::GearAudio;
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::types::evidence::Evidence;
use unsound_core::resources::SoundGrid;
use unspatial_core::position::Position;

use bevy::prelude::*;
use rand::Rng as _;
use unfoundation_core::types::gear::EquipmentPosition;
use ungear_core::components::core::{
    Battery, Electronic, GearSprite, PerceivedClarity, StatusText,
};
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::geigercounter::GeigerCounter;
use uninteraction_core::interaction::Toggleable;
use unrender_std::resources::sprite_registry::GearSpriteID;

pub(crate) trait GeigerCounterExt {
    fn calculate_output_sound(&self, haunt_state: &HauntState) -> f32;
}

impl GeigerCounterExt for GeigerCounter {
    fn calculate_output_sound(&self, haunt_state: &HauntState) -> f32 {
        let sum_snd: f32 = self.sound_l.iter().sum();
        let avg_snd: f32 = sum_snd / self.sound_l.len() as f32;
        let evidence = haunt_state.ghost_dynamics.cpm500_clarity.cbrt().max(-0.05);

        f32::tanh(avg_snd.sqrt() / (10.0 + evidence * 2.0)) * (480.0 + evidence * 500.0)
    }
}

pub(crate) fn update_geigercounter(
    mut q_geiger: Query<(
        &mut GeigerCounter,
        &mut StatusText,
        &mut GearSprite,
        &mut Toggleable,
        &mut Battery,
        &Electronic,
        &Position,
        &EquipmentPosition,
        &mut PerceivedClarity,
    )>,
    mut gs_audio: GearAudio,
    sg: Res<SoundGrid>,
    difficulty: Res<CurrentDifficulty>,
    haunt_state: Res<HauntState>,
) {
    for (
        mut geiger,
        mut status,
        mut sprite,
        toggle,
        mut battery,
        electronic,
        pos,
        _ep,
        mut perceived_clarity,
    ) in q_geiger.iter_mut()
    {
        let mut rng = random_seed::rng();
        geiger.display_secs_since_last_update += gs_audio.time.delta_secs(); // Increment the timer
        geiger.frame_counter += 1;
        geiger.frame_counter %= 65413;

        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        const K: f32 = 0.5;
        let posk = Position {
            x: pos.x + rng.random_range(-K..K) + rng.random_range(-K..K),
            y: pos.y + rng.random_range(-K..K) + rng.random_range(-K..K),
            z: pos.z,
            visual_priority: pos.visual_priority,
        };
        let dist2breach = haunt_state.breach_pos.distance2(&posk) + 10.0;
        let breach_energy = dist2breach.recip() * 20000.0;
        let bpos = posk.to_board_position();
        for (i, bpos) in bpos.iter_xy_neighbors_nosize(4).enumerate() {
            let sound = sg.sound_field.get(&bpos).cloned().unwrap_or_default();
            let sound_reading = sound.iter().sum::<Vec2>().length() * 1000.0;
            if geiger.sound_l.len() < 1200 {
                geiger.sound_l.push(sound_reading);
            }
            let n = (geiger.frame_counter as usize + i) % geiger.sound_l.len();
            geiger.sound_l[n] /= 4.0 * difficulty.0.equipment_sensitivity;
            if toggle.is_on {
                geiger.sound_l[n] +=
                    sound_reading * 40.0 + breach_energy * difficulty.0.equipment_sensitivity;
            }
        }

        geiger.sound_l.iter_mut().for_each(|x| *x /= 1.06);

        let mass: f32 = 8.0 * difficulty.0.equipment_sensitivity;
        if toggle.is_on {
            // Calculate the *current* output sound.
            let current_output_sound = geiger.calculate_output_sound(&haunt_state);
            // Smooth the *current* output to get sound_a1 (first IIR filter).
            geiger.sound_a1 = (geiger.sound_a1 * mass + current_output_sound * mass.recip())
                / (mass + mass.recip());

            let mass = mass
                * if current_output_sound > geiger.sound_a2 {
                    1.0
                } else {
                    2.0
                };
            // Smooth sound_a1 to get output_sound (second IIR filter).
            geiger.output_sound = (geiger.output_sound * mass + geiger.sound_a1 * mass.recip())
                / (mass + mass.recip());

            geiger.sound_a2 =
                (geiger.sound_a2 * mass + geiger.sound_a1 * mass.recip()) / (mass + mass.recip());
        } else {
            geiger.sound_a1 /= 1.01;
            geiger.sound_a2 /= 1.01;
        }

        if gs_audio.time.elapsed_secs() - geiger.last_sound_time_secs > 60.0 / geiger.sound_a1
            && toggle.is_on
        {
            if electronic.glitch_timer <= 0.0001 {
                geiger.last_sound_time_secs =
                    gs_audio.time.elapsed_secs() + rng.random_range(0.01..0.02);
                gs_audio.play_audio("sounds/effects-chirp-click.ogg".into(), 0.25, pos);
            } else {
                geiger.last_sound_time_secs =
                    gs_audio.time.elapsed_secs() + rng.random_range(0.01..0.02);
                gs_audio.play_audio("sounds/effects-chirp-short.ogg".into(), 0.25, pos);
            }
        }
        // Update sound_display *only* if enough time has passed.
        if geiger.display_secs_since_last_update > 0.5 {
            geiger.display_secs_since_last_update = 0.0; // Reset the timer
            geiger.sound_display = geiger.output_sound; // Update the display value

            // Update blinking_hint_active
            const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
            // Consider evidence showing if cpm is >= 500 and not glitching
            if geiger.sound_display >= 499.9 && electronic.glitch_timer <= 0.0 {
                let count = gs_audio
                    .player_profile
                    .times_evidence_acknowledged_on_gear
                    .get(&Evidence::CPM500)
                    .copied()
                    .unwrap_or(0);
                geiger.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
            } else {
                geiger.blinking_hint_active = false;
            }
        } else {
            // Ensure blinking_hint_active is false if not updating display this frame,
            // or if we want it to strictly follow the evidence condition.
            if !(geiger.sound_display >= 499.9 && electronic.glitch_timer <= 0.0) {
                geiger.blinking_hint_active = false;
            }
        }

        // Update StatusText
        let name = "Geiger Counter";
        let on_s = on_off(toggle.is_on);

        // Show garbled text when enabled (intent) but glitching (actual state)
        if toggle.is_on && electronic.glitch_timer > 0.0 {
            let garbled = match rng.random_range(0..4) {
                0 => "Reading: ERR0R\nEnergy: ###.###",
                1 => "Reading: ---.--\nEnergy: FAULT",
                2 => "INTERFERENCE DET---\nCALIBRATING...",
                _ => "Signal Lost\nReacquiring...",
            };
            status.0 = format!("{name}:  {on_s}\n{garbled}");
        } else {
            let msg = if toggle.is_on && electronic.glitch_timer <= 0.0 {
                let cpm_text = format!("{:.1}", geiger.sound_display);
                if geiger.blinking_hint_active {
                    let blinking_cpm_text = if geiger.frame_counter % 30 < 15 {
                        format!(">[{}]<", cpm_text)
                    } else {
                        format!("  {}  ", cpm_text)
                    };
                    format!("Reading: {}cpm", blinking_cpm_text)
                } else {
                    format!("Reading: {}cpm", cpm_text)
                }
            } else {
                "".to_string()
            };
            status.0 = format!("{name}: {on_s}\n{msg}");
        }

        // Update GearSprite
        if toggle.is_on {
            if electronic.glitch_timer > 0.0 {
                // Glitching: flicker between Off and Tick
                if rng.random_bool(0.7) {
                    sprite.0 = GearSpriteID::GeigerOff.to_visual_key();
                } else {
                    sprite.0 = GearSpriteID::GeigerTick.to_visual_key();
                }
            } else if geiger.sound_a1 > 10.0 {
                sprite.0 = GearSpriteID::GeigerTick.to_visual_key();
            } else {
                sprite.0 = GearSpriteID::GeigerOn.to_visual_key();
            }
        } else {
            sprite.0 = GearSpriteID::GeigerOff.to_visual_key();
        }

        perceived_clarity.from_status_text =
            if toggle.is_on && geiger.sound_display > 500.0 && electronic.glitch_timer <= 0.0 {
                1.0
            } else {
                0.0
            };
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_geigercounter);
}
