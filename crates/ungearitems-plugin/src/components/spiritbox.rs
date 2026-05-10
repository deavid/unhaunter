use unaudiospatial_core::emitter::LocalAudioEmitter;
use uncommon_app_core::random_seed;
use ungear_core::components::core::{
    Battery, Electronic, GearSprite, ItemName, PerceivedClarity, StatusText,
};
use unsoundfield_core::resources::SoundGrid;
use unthermal_core::resources::ThermalGrid;

#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct SpiritBoxInternal {
    pub last_response_time: Option<f64>,
}

use uninteraction_core::interaction::Toggleable;

use bevy::prelude::*;
use bevy_persistent::Persistent;
use rand::RngExt;
use uncommon_app_core::utils::temperature::kelvin_to_celsius;
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::spiritbox::SpiritBox;
use unghost_core::components::logic::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use uninvestigation_core::evidence::Evidence;
use unlight_core::resources::light_grid::LightGrid;
use unmetrics_core::metrics::SendMetric;
use unprofile_core::profile::PlayerProfileData;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::position::Position;

use crate::metrics;

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
    mut gs_audio: LocalAudioEmitter,
    tg: If<Res<ThermalGrid>>,
    sg: If<Res<SoundGrid>>,
    q_ghost: Query<(&GhostSprite, &Position, &GhostBehaviorDynamics)>,
    lg: If<Res<LightGrid>>,
    player_profile: If<Res<Persistent<PlayerProfileData>>>,
    mut commands: Commands,
) {
    let measure = metrics::SPIRITBOX_UPDATE.time_measure();
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
        let is_glitching = electronic.glitch_timer > 0.0;

        // Update Battery Drain Rate
        battery.drain_rate = if toggle.is_on { 0.0001 } else { 0.0 };

        if !toggle.is_on {
            spiritbox.blinking_hint_active = false;
            sprite.0 = GearSpriteID::SpiritBoxOff.to_visual_key();
            status.0 = format!("{}: {}", name.0, on_off(toggle.is_on));
            continue;
        }

        let delta = sec - spiritbox.last_change_secs;

        // Unified Tick Logic (4Hz or 0.25s)
        if delta >= 0.25 {
            spiritbox.last_change_secs = sec;

            // Wait out the answer
            if spiritbox.ghost_answer {
                if delta >= 3.0 {
                    spiritbox.ghost_answer = false;
                    spiritbox.blinking_hint_active = false;
                    // Reset timer so we immediately start scanning next frame
                    spiritbox.last_change_secs = sec;
                }
            } else if !is_glitching {
                // In Scanning Mode
                spiritbox.mode_frame = spiritbox.mode_frame.wrapping_add(1);

                let bpos = pos.to_board_position();
                let temperature = tg.temperature_field[bpos.ndidx()];
                let temp_c = kelvin_to_celsius(temperature);
                let light_lux = lg
                    .light_field
                    .get(bpos.ndidx())
                    .cloned()
                    .unwrap_or_default()
                    .lux;

                let mut ghost_near = false;
                let mut spiritbox_clarity = 0.0;
                for (ghost, ghost_pos, dynamics) in q_ghost.iter() {
                    if ghost.hunting > 0.0 {
                        continue;
                    }
                    let dist2 = pos.distance2(ghost_pos);
                    if dist2 < 3.0 * 3.0 && ghost.class.evidences().contains(&Evidence::SpiritBox) {
                        ghost_near = true;
                        spiritbox_clarity = dynamics.spirit_box_clarity;
                        break;
                    }
                }

                // Accumulate charge
                if ghost_near {
                    let sound = sg.sound_field.get(&bpos).cloned().unwrap_or_default();
                    let sound_reading = sound.iter().sum::<Vec2>().length() * 100.0;
                    let light_clamped = (light_lux * 5.0).clamp(0.3, 10.0);
                    let temp_clamped = (temp_c - 3.0).clamp(0.5, 10.0);
                    spiritbox.charge += sound_reading / temp_clamped.powi(2) / light_clamped / 5.5
                        * spiritbox_clarity.max(0.0);
                }

                // Check for generic scan tick or ghost answer
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
                        _ => spiritbox.ghost_answer = false,
                    }

                    // Update blinking_hint_active
                    const HINT_ACKNOWLEDGE_THRESHOLD: u32 = 3;
                    let count = player_profile
                        .times_evidence_acknowledged_on_gear
                        .get(&Evidence::SpiritBox)
                        .copied()
                        .unwrap_or(0);
                    spiritbox.blinking_hint_active = count < HINT_ACKNOWLEDGE_THRESHOLD;
                } else {
                    // Play regular scan sound (every tick, 4 times per sec)
                    gs_audio.play_audio("sounds/effects-radio-scan.ogg".into(), 0.2, pos);
                }
            }
        }

        // Handle Glitching audio overrides
        if is_glitching && rng.random_range(0.0..1.0) < 0.05 {
            gs_audio.play_audio("sounds/effects-chirp-click.ogg".into(), 0.5, pos);
        }

        // Update Sprite
        sprite.0 = if is_glitching {
            match rng.random_range(0..5) {
                0 => GearSpriteID::SpiritBoxOff.to_visual_key(),
                1 => GearSpriteID::SpiritBoxScan1.to_visual_key(),
                2 => GearSpriteID::SpiritBoxScan2.to_visual_key(),
                3 => GearSpriteID::SpiritBoxScan3.to_visual_key(),
                _ => GearSpriteID::SpiritBoxAns1.to_visual_key(),
            }
        } else if spiritbox.ghost_answer {
            // Visual toggle based on time for Answer mode (blinking effect)
            if ((sec * 4.0) as u32).is_multiple_of(2u32) {
                GearSpriteID::SpiritBoxAns1.to_visual_key()
            } else {
                GearSpriteID::SpiritBoxAns2.to_visual_key()
            }
        } else {
            // Visual scanning
            match spiritbox.mode_frame % 3 {
                0 => GearSpriteID::SpiritBoxScan1.to_visual_key(),
                1 => GearSpriteID::SpiritBoxScan2.to_visual_key(),
                _ => GearSpriteID::SpiritBoxScan3.to_visual_key(),
            }
        };

        // Update Status Text
        let on_s = on_off(toggle.is_on);

        let msg = if is_glitching {
            match rng.random_range(0..5) {
                0 => "Signal: --LOST--",
                1 => "Static....",
                2 => "....?--?---",
                3 => "MESSAG? IMPOSSI-",
                _ => "CHAOTIC SIGNALS",
            }
        } else if spiritbox.ghost_answer {
            if spiritbox.blinking_hint_active && ((sec * 4.0) as u32).is_multiple_of(2) {
                "> EVP Detected! <"
            } else {
                "  EVP Detected!  "
            }
        } else {
            "Scanning.."
        };

        status.0 = format!("{}: {}\n{}", name.0, on_s, msg);

        if spiritbox.ghost_answer {
            internal.last_response_time = Some(gs_audio.time.elapsed_secs_f64());
        }

        let is_recent_response = internal
            .last_response_time
            .is_some_and(|t| gs_audio.time.elapsed_secs_f64() - t < 10.0);

        perceived_clarity.from_sound = if is_recent_response && electronic.glitch_timer <= 0.0 {
            1.0
        } else {
            0.0
        };
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_spiritbox.run_if(resource_exists::<LocalPlayerRole>),
    );
}
