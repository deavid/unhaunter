use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::random_seed;
use ungear_core::components::core::{GearSprite, ItemName, PerceivedClarity, StatusText};
use ungear_core::gear_stuff::GearAudio;
use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::Toggleable;
use unsound_core::resources::SoundGrid;

use bevy::prelude::*;
use rand::Rng;
use ungear_core::types::gear::utils::on_off;
use ungearitems_core::components::recorder::Recorder;
use unghost_core::types::evidence::Evidence;
use unrender_std::resources::sprite_registry::GearSpriteID;
use unspatial_core::position::Position;

pub(crate) fn update_recorder(
    mut q_recorder: Query<(
        &mut Recorder,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &Position,
        &ItemName,
        &mut PerceivedClarity,
    )>,
    mut gs_audio: GearAudio,
    sg: Res<SoundGrid>,
    haunt_state: Res<HauntState>,
    difficulty: Res<CurrentDifficulty>,
) {
    for (mut recorder, mut status, mut sprite, toggle, pos, name, mut perceived_clarity) in
        q_recorder.iter_mut()
    {
        let mut rng = random_seed::rng();
        recorder.frame_counter = recorder.frame_counter.wrapping_add(1);

        // Update Sprite
        sprite.0 = if !toggle.is_on {
            GearSpriteID::RecorderOff.to_visual_key()
        } else if recorder.display_glitch_timer > 0.0 && rng.random_range(0.0..1.0) < 0.4 {
            match rng.random_range(0..3) {
                0 => GearSpriteID::RecorderOff.to_visual_key(),
                1 => GearSpriteID::Recorder4.to_visual_key(), // Show max reading
                _ => GearSpriteID::Recorder1.to_visual_key(),
            }
        } else {
            // Normal operation
            let f = rng.random_range(0.5..2.0);

            // Add artificial noise during false readings
            let mut s = recorder.sound;
            if recorder.false_reading_timer > 0.0 {
                s = s * 2.5 + 30.0 * recorder.false_reading_timer;
            }
            s *= f;

            if s < 5.0 {
                GearSpriteID::Recorder1.to_visual_key()
            } else if s < 15.0 {
                GearSpriteID::Recorder2.to_visual_key()
            } else if s < 45.0 {
                GearSpriteID::Recorder3.to_visual_key()
            } else {
                GearSpriteID::Recorder4.to_visual_key()
            }
        };

        // Update Logic
        if toggle.is_on {
            let bpos = pos.to_board_position();
            let sound = sg.sound_field.get(&bpos).cloned().unwrap_or_default();
            let sound_reading = sound.iter().sum::<Vec2>().length() * 1000.0;

            recorder.sound_l.push(sound_reading);
            if recorder.sound_l.len() > 100 {
                recorder.sound_l.remove(0);
            }

            let dt = gs_audio.time.delta_secs();
            recorder.display_secs_since_last_update += dt;
            if recorder.display_secs_since_last_update > 0.5 {
                recorder.display_secs_since_last_update = 0.0;
                let sum: f32 = recorder.sound_l.iter().sum();
                recorder.sound = sum / recorder.sound_l.len() as f32;
            }

            let mut evp_recorded = false;
            if let Some(ghost_pos) = haunt_state.ghost_warning_position {
                let dist2 = pos.distance2(&ghost_pos);
                if dist2 < 2.0 * 2.0 {
                    evp_recorded = true;
                }
            }

            if evp_recorded {
                recorder.amt_recorded += dt * difficulty.0.equipment_sensitivity;
            } else {
                recorder.amt_recorded -= dt * 0.1;
            }
            recorder.amt_recorded = recorder.amt_recorded.clamp(0.0, 20.0);

            if recorder.amt_recorded > 10.0 {
                recorder.evp_recorded_time_secs = 10.0;
                recorder.evp_recorded_count += 1;
                recorder.amt_recorded = 0.0;

                // Update blinking_hint_active
                const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                let count = gs_audio
                    .player_profile
                    .times_evidence_acknowledged_on_gear
                    .get(&Evidence::EVPRecording)
                    .copied()
                    .unwrap_or(0);
                recorder.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
            }

            if recorder.evp_recorded_time_secs > 0.0 {
                recorder.evp_recorded_time_secs -= dt;
                recorder.evp_recorded_display = true;
            } else {
                recorder.evp_recorded_display = false;
                recorder.blinking_hint_active = false;
            }

            // Decrement glitch timer if active
            if recorder.display_glitch_timer > 0.0 {
                recorder.display_glitch_timer -= dt;

                // Play static/interference sound when glitching
                if rng.random_range(0.0..1.0) < 0.4 {
                    gs_audio.play_audio("sounds/effects-chirp-short.ogg".into(), 0.3, pos);
                }
            }

            // Decrement false reading timer
            if recorder.false_reading_timer > 0.0 {
                recorder.false_reading_timer -= dt;
            }

            // Apply EMI if warning is active and we're electronic
            if let Some(ghost_pos) = &haunt_state.ghost_warning_position {
                let distance2 = pos.distance2(ghost_pos);
                let warning_level = haunt_state.ghost_warning_intensity;
                if warning_level > 0.0001 {
                    // Scale effect by distance and warning level
                    let effect_strength = warning_level * (100.0 / distance2).min(1.0);

                    // Random display glitches
                    if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
                        recorder.display_glitch_timer = 0.4;
                    }

                    // Random false audio spikes
                    if rng.random_range(0.0..1.0) < effect_strength.powi(3) * 0.5 {
                        recorder.false_reading_timer = rng.random_range(0.5..2.0);
                    }
                }
            }
        }

        // Update Status Text
        let on_s = on_off(toggle.is_on);

        // Show garbled text when glitching
        if toggle.is_on && recorder.display_glitch_timer > 0.0 {
            let garbled = match rng.random_range(0..5) {
                0 => "Vol: ****ERROR****",
                1 => "INTERFERENCE DETE---",
                2 => "--EVP D??E*T?D--",
                3 => "SIGNAL:NOISE=0.---",
                _ => "AUDIO MALFUNCTION",
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, garbled);
            continue;
        }

        // Normal display
        let msg = if toggle.is_on {
            if recorder.evp_recorded_display {
                // This state implies evidence has been found and is being actively displayed
                if recorder.blinking_hint_active {
                    if recorder.frame_counter % 30 < 15 {
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
                    recorder.sound - 93.0,
                    recorder.evp_recorded_count
                )
            }
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);

        perceived_clarity.from_status_text = if toggle.is_on
            && recorder.evp_recorded_count > 0
            && recorder.display_glitch_timer <= 0.0
        {
            1.0
        } else {
            0.0
        };
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_recorder);
}
