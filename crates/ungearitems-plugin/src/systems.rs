use bevy::prelude::*;
use rand::prelude::*;
use unfoundation_core::random_seed;
use ungear_core::components::core::{Battery, Electronic};
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::gear_details::GearDetails;
use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::{Toggleable, Triggered};
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::{MainPlayer, PlayerInput};
use unaudiospatial_core::emitter::AudioEmitter;
use unspatial_core::position::Position;

use crate::components::flashlight::Flashlight;
use crate::components::repellentflask::RepellentFlask;
use crate::metrics;

pub(crate) fn system_electronic_interference(
    gs_audio: AudioEmitter,
    haunt_state: Res<HauntState>,
    mut q_electronic: Query<(&Position, &mut Electronic, &Toggleable)>,
) {
    let measure = metrics::ELECTRONIC_INTERFERENCE.time_measure();
    let mut rng = random_seed::rng();
    let dt = gs_audio.time.delta_secs();

    for (pos, mut electronic, toggle) in q_electronic.iter_mut() {
        // Decrement glitch timer if active
        if electronic.glitch_timer > 0.0 {
            electronic.glitch_timer -= dt;
        }

        // Apply EMI if warning is active and item is on
        if let Some(ghost_pos) = &haunt_state.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            if haunt_state.ghost_warning_intensity > 0.0001 && toggle.is_on {
                // Scale effect by distance and warning level
                let effect_strength = haunt_state.ghost_warning_intensity
                    * (100.0 / distance2).min(1.0)
                    * electronic.sensitivity;

                electronic.glitch_intensity = effect_strength;

                // Random glitches
                if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
                    electronic.glitch_timer = rng.random_range(0.2..0.6);
                }
            } else {
                electronic.glitch_intensity = 0.0;
            }
        } else {
            electronic.glitch_intensity = 0.0;
        }
    }

    measure.end_ms();
}

pub(crate) fn system_battery_drain(
    time: Res<Time>,
    mut q_battery: Query<(&mut Battery, &mut Toggleable)>,
) {
    let measure = metrics::BATTERY_DRAIN.time_measure();
    let dt = time.delta_secs();

    for (mut battery, mut toggle) in q_battery.iter_mut() {
        if toggle.is_on {
            battery.level -= battery.drain_rate * dt;
            if battery.level <= 0.0 {
                battery.level = 0.0;
                toggle.is_on = false; // Auto-shutdown
            }
        }
    }

    measure.end_ms();
}

/// Applies gear state from `PlayerInput.target_*_hand` to gear components.
/// Only runs for non-MainPlayer entities (remote players on the host).
/// The client sends its gear state every frame; the host copies it here.
pub(crate) fn system_apply_gear_intent_from_input(
    mut commands: Commands,
    q_players: Query<(&PlayerGear, &PlayerInput), Without<MainPlayer>>,
    mut q_toggleable: Query<&mut Toggleable>,
    mut q_flashlight: Query<(&mut Flashlight, &Battery)>,
    mut q_sage: Query<&mut ungearitems_core::components::sage::SageBundleData>,
    mut q_repellent: Query<&mut RepellentFlask>,
) {
    for (pg, pi) in q_players.iter() {
        // Right hand — apply full state (not just on click)
        if let Some((on, details)) = &pi.target_right_hand
            && let Some(entity) = pg.right_hand
        {
            if let Ok(mut toggle) = q_toggleable.get_mut(entity) {
                toggle.is_on = *on;
            }
            match details {
                GearDetails::Flashlight(status) => {
                    if let Ok((mut flashlight, battery)) = q_flashlight.get_mut(entity) {
                        if flashlight.can_enable_status(status.clone(), battery.level) {
                            flashlight.status = status.clone();
                        } else {
                            flashlight.status =
                                ungearitems_core::components::flashlight::FlashlightStatus::Off;
                        }
                        commands.entity(entity).remove::<Triggered>();
                    }
                }
                GearDetails::Sage { is_active, .. } => {
                    if let Ok(mut sage) = q_sage.get_mut(entity) {
                        if *is_active && !sage.is_active && !sage.consumed {
                            sage.is_active = true;
                        }
                        commands.entity(entity).remove::<Triggered>();
                    }
                }
                GearDetails::RepellentFlask { active, .. } => {
                    if let Ok(mut repellent) = q_repellent.get_mut(entity) {
                        repellent.active = *active;
                        commands.entity(entity).remove::<Triggered>();
                    }
                }
                _ => {}
            }
        }
        // Left hand — apply full state (not just on click)
        if let Some((on, details)) = &pi.target_left_hand
            && let Some(entity) = pg.left_hand
        {
            if let Ok(mut toggle) = q_toggleable.get_mut(entity) {
                toggle.is_on = *on;
            }
            match details {
                GearDetails::Flashlight(status) => {
                    if let Ok((mut flashlight, battery)) = q_flashlight.get_mut(entity) {
                        if flashlight.can_enable_status(status.clone(), battery.level) {
                            flashlight.status = status.clone();
                        } else {
                            flashlight.status =
                                ungearitems_core::components::flashlight::FlashlightStatus::Off;
                        }
                        commands.entity(entity).remove::<Triggered>();
                    }
                }
                GearDetails::Sage { is_active, .. } => {
                    if let Ok(mut sage) = q_sage.get_mut(entity) {
                        if *is_active && !sage.is_active && !sage.consumed {
                            sage.is_active = true;
                        }
                        commands.entity(entity).remove::<Triggered>();
                    }
                }
                GearDetails::RepellentFlask { active, .. } => {
                    if let Ok(mut repellent) = q_repellent.get_mut(entity) {
                        repellent.active = *active;
                        commands.entity(entity).remove::<Triggered>();
                    }
                }
                _ => {}
            }
        }
    }
}
