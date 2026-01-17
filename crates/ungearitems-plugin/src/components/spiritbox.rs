use unfoundation_core::random_seed;
use ungear_core::components::core::{
    Battery, Electronic, GearSprite, ItemName, PerceivedClarity, StatusText,
};
use ungear_core::gear_stuff::{GearAudio, GearGameState, GearResources};

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct SpiritBoxInternal {
    pub last_response_time: Option<f64>,
}

use uninteraction_core::interaction::Toggleable;

use bevy::prelude::*;
use rand::Rng;
use unfoundation_core::types::evidence::Evidence;
use unfoundation_core::types::gear::GearSpriteID;
use unfoundation_core::utils::kelvin_to_celsius;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::spiritbox::SpiritBox;
use unlight_plugin::resources::light_grid::LightGrid;
use unspatial_core::position::Position;

pub(crate) fn update_spiritbox(
    mut q_spiritbox: Query<(
        Entity,
        &mut SpiritBox,
        &mut StatusText,
        &mut GearSprite,
        &Toggleable,
        &mut Battery,
        &Electronic,
        &Position,
        &ItemName,
        &mut PerceivedClarity,
        Option<&mut SpiritBoxInternal>,
    )>,
    mut gs_audio: GearAudio,
    gs_res: GearResources,
    gs_state: GearGameState,
    lg: Res<LightGrid>,
    mut commands: Commands,
) {
    for (
        entity,
        mut spiritbox,
        mut status,
        mut sprite,
        toggle,
        mut battery,
        electronic,
        pos,
        name,
        mut perceived_clarity,
        mut internal_opt,
    ) in q_spiritbox.iter_mut()
    {
        let internal = if let Some(i) = internal_opt.as_deref_mut() {
            i
        } else {
            commands.entity(entity).insert(SpiritBoxInternal::default());
            continue;
        };

        let mut rng = random_seed::rng();
        let sec = gs_audio.time.elapsed_secs();
        spiritbox.mode_frame = (sec * 4.0).round() as u32;

        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        // Update Sprite
        sprite.0 = if electronic.glitch_timer > 0.0 {
            match rng.random_range(0..5) {
                0 => GearSpriteID::SpiritBoxOff,   // Blank/off
                1 => GearSpriteID::SpiritBoxScan1, // Flickering
                2 => GearSpriteID::SpiritBoxScan2,
                3 => GearSpriteID::SpiritBoxScan3,
                _ => GearSpriteID::SpiritBoxAns1, // Maybe show as if it answered
            }
        } else if toggle.is_on {
            if spiritbox.ghost_answer {
                match spiritbox.mode_frame % 2 {
                    0 => GearSpriteID::SpiritBoxAns1,
                    _ => GearSpriteID::SpiritBoxAns2,
                }
            } else {
                match spiritbox.mode_frame % 3 {
                    0 => GearSpriteID::SpiritBoxScan1,
                    1 => GearSpriteID::SpiritBoxScan2,
                    _ => GearSpriteID::SpiritBoxScan3,
                }
            }
        } else {
            GearSpriteID::SpiritBoxOff
        };

        // Update Logic
        if toggle.is_on {
            let bpos = pos.to_board_position();
            let temperature = gs_res.tg.temperature_field[bpos.ndidx()];
            let temp_c = kelvin_to_celsius(temperature);
            let light_lux = lg
                .light_field
                .get(bpos.ndidx())
                .cloned()
                .unwrap_or_default()
                .lux;

            let mut ghost_near = false;
            if let Some(ghost_pos) = gs_state.haunt_state.ghost_warning_position {
                let dist2 = pos.distance2(&ghost_pos);
                if dist2 < 3.0 * 3.0 {
                    ghost_near = true;
                }
            }

            let delta = sec - spiritbox.last_change_secs;

            // Only charge up for a response if the ghost has the Spirit Box evidence.
            if gs_state
                .haunt_state
                .evidences
                .contains(&Evidence::SpiritBox)
                && ghost_near
            {
                let sound = gs_res
                    .sg
                    .sound_field
                    .get(&bpos)
                    .cloned()
                    .unwrap_or_default();
                let sound_reading = sound.iter().sum::<Vec2>().length() * 100.0;
                let light_clamped = (light_lux * 5.0).clamp(0.3, 10.0);
                let temp_clamped = (temp_c - 3.0).clamp(0.5, 10.0);
                spiritbox.charge += sound_reading / temp_clamped.powi(2) / light_clamped / 15.0
                    * gs_state
                        .haunt_state
                        .ghost_dynamics
                        .spirit_box_clarity
                        .max(0.0);
            }

            if spiritbox.ghost_answer {
                if delta > 3.0 {
                    spiritbox.ghost_answer = false;
                    spiritbox.blinking_hint_active = false;
                }
            } else if delta > 0.3 && electronic.glitch_timer <= 0.0 {
                spiritbox.last_change_secs = sec;
                gs_audio.play_audio("sounds/effects-radio-scan.ogg".into(), 0.4, pos);

                let r = if spiritbox.charge > 30.0 {
                    spiritbox.charge = 0.0;
                    rng.random_range(0..10)
                } else {
                    99 // Not enough charge, no answer
                };

                spiritbox.ghost_answer = matches!(r, 0..=3);

                if spiritbox.ghost_answer {
                    match r {
                        0 => {
                            gs_audio.play_audio("sounds/effects-radio-answer1.ogg".into(), 0.7, pos)
                        }
                        1 => {
                            gs_audio.play_audio("sounds/effects-radio-answer2.ogg".into(), 0.7, pos)
                        }
                        2 => {
                            gs_audio.play_audio("sounds/effects-radio-answer3.ogg".into(), 0.7, pos)
                        }
                        3 => {
                            gs_audio.play_audio("sounds/effects-radio-answer4.ogg".into(), 0.4, pos)
                        }
                        _ => spiritbox.ghost_answer = false, // Should not happen, but safeguard.
                    }

                    // Update blinking_hint_active
                    const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                    let count = gs_audio
                        .player_profile
                        .times_evidence_acknowledged_on_gear
                        .get(&Evidence::SpiritBox)
                        .copied()
                        .unwrap_or(0);
                    spiritbox.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
                }
            } else if delta > 0.3 && electronic.glitch_timer > 0.0 {
                spiritbox.last_change_secs = sec;
                gs_audio.play_audio("sounds/effects-radio-scan.ogg".into(), 0.4, pos);
            }

            // Play more static sounds when glitching
            if electronic.glitch_timer > 0.0 && rng.random_range(0.0..1.0) < 0.6 {
                gs_audio.play_audio("sounds/effects-chirp-click.ogg".into(), 0.5, pos);
            }

            // Play scanning sound
            if !spiritbox.ghost_answer && spiritbox.mode_frame % 10 == 0 {
                gs_audio.play_audio("sounds/effects-radio-scan.ogg".into(), 0.1, pos);
            }
            if spiritbox.ghost_answer && spiritbox.mode_frame % 20 == 0 {
                gs_audio.play_audio("sounds/effects-radio-scan.ogg".into(), 0.4, pos);
            }
        } else {
            // Ensure hint is off when disabled
            spiritbox.blinking_hint_active = false;
        }

        // Update Status Text
        let on_s = on_off(toggle.is_on);

        // Glitch text
        if toggle.is_on && electronic.glitch_timer > 0.0 {
            let garbled = match rng.random_range(0..5) {
                0 => "Signal: --LOST--",
                1 => "Static....",
                2 => "....?--?---",
                3 => "MESSAG? IMPOSSI-",
                _ => "CHAOTIC SIGNALS",
            };
            status.0 = format!("{}: {}\n{}", name.0, on_s, garbled);
            continue;
        }

        // Normal status
        let msg = if toggle.is_on {
            if spiritbox.ghost_answer {
                if spiritbox.blinking_hint_active {
                    if spiritbox.mode_frame % 20 < 10 {
                        // Blinking effect
                        "> EVP Detected! <".to_string()
                    } else {
                        "  EVP Detected!  ".to_string()
                    }
                } else {
                    "EVP Detected!".to_string()
                }
            } else {
                "Scanning..".to_string()
            }
        } else {
            "".to_string()
        };
        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);

        if spiritbox.ghost_answer {
            internal.last_response_time = Some(gs_audio.time.elapsed_secs_f64());
        }

        let is_recent_response = internal
            .last_response_time
            .is_some_and(|t| gs_audio.time.elapsed_secs_f64() - t < 10.0);

        perceived_clarity.from_sound =
            if toggle.is_on && is_recent_response && electronic.glitch_timer <= 0.0 {
                1.0
            } else {
                0.0
            };
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, update_spiritbox);
}
