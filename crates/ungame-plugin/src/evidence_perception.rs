use bevy::prelude::*;
use unlight_plugin::resources::light_grid::LightGrid;
use unfoundation_core::types::evidence::Evidence;
use unfoundation_core::types::light::LightType;
use ungear_core::components::core::{Electronic, EvidenceSensor, PerceivedClarity};
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::looking_gear::LookingGear;
use unghost_core::components::GhostOrbParticle;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use unghost_core::resources::haunt_state::HauntState;
use uninteraction_core::interaction::Toggleable;
use unrender_std::components::light::LightEmitter;
use unspatial_core::position::Position;
use untags_core::tags::PlayerTag;

fn update_current_evidence_readings_from_player_perception_system(
    mut evidence_readings: ResMut<CurrentEvidenceReadings>,
    player_query: Query<(&PlayerGear, &Position), With<PlayerTag>>,
    looking_gear: Res<LookingGear>,
    q_evidence_sensor: Query<(
        &EvidenceSensor,
        &Toggleable,
        &PerceivedClarity,
        Option<&Electronic>,
    )>,
    q_deployed_gear: Query<(Entity, &Position, &DeployedGear)>,
    q_ghost: Query<(&GhostSprite, &Position)>,
    q_orb: Query<&Position, With<GhostOrbParticle>>,
    q_light: Query<(&LightEmitter, &Toggleable, &Position)>,
    light_grid: Res<LightGrid>,
    haunt_state: Res<HauntState>,
    time: Res<Time>,
) {
    let Some((player_gear, player_pos)) = player_query.iter().next() else {
        return;
    };

    let delta_time = time.delta_secs();
    let current_time = time.elapsed_secs_f64();

    // Check if any ghost is currently hunting (map-wide)
    let is_any_ghost_hunting = q_ghost
        .iter()
        .any(|(g, _)| g.hunting > 0.0 || g.hunt_target);

    // Helper closure to process a single piece of gear
    let mut process_gear = |entity: Entity, status_vis: bool, icon_vis: bool, sound_vis: bool| {
        if let Ok((sensor, toggle, clarity, electronic)) = q_evidence_sensor.get(entity) {
            if !toggle.is_on {
                return;
            }

            let evidence = sensor.evidence;
            let mut c = 0.0f32;

            // If a hunt is active anywhere, electronic equipment cannot gather reliable evidence.
            // Also check for local glitching.
            if is_any_ghost_hunting
                || electronic.is_some_and(|e| e.glitch_intensity > 0.5 || e.glitch_timer > 0.1)
            {
                c = 0.0;
            } else {
                if status_vis {
                    c = c.max(clarity.from_status_text);
                }
                if icon_vis {
                    c = c.max(clarity.from_icon);
                }
                if sound_vis {
                    c = c.max(clarity.from_sound);
                }

                // Environmental fallbacks / Special Conditions
                if evidence == Evidence::FloatingOrbs {
                    // Floating orbs are visible if lights are off and actual orbs are visible near the player/camera.
                    let bpos = player_pos.to_board_position();
                    let lux = light_grid
                        .light_field
                        .get(bpos.ndidx())
                        .map(|l| l.lux)
                        .unwrap_or(0.0);

                    // Video camera allows seeing orbs if dark or NV is on (NV handled by gear enabled)
                    // This block runs if we have a sensor (VideoCam) enabled.
                    // We check if ambient light is low enough for orbs to be distinct, or if using NV.
                    // We iterate over all orb particles.
                    const ORB_VISIBILITY_RANGE: f32 = 8.0;
                    let mut orbs_visible = 0;
                    for orb_pos in q_orb.iter() {
                        if orb_pos.distance(player_pos) < ORB_VISIBILITY_RANGE {
                            orbs_visible += 1;
                        }
                    }
                    if orbs_visible > 0 && lux < 0.5 {
                        c = c.max(1.0);
                    }
                }
            }

            evidence_readings.report_clarity(evidence, c.clamp(0.0, 1.0), current_time, delta_time);
        }
    };

    // --- Right Hand ---
    if let Some(e) = player_gear.right_hand {
        process_gear(e, true, true, true);
    }

    // --- Left Hand ---
    if let Some(e) = player_gear.left_hand {
        // Status text only visible if player is holding the "look at left hand" key
        process_gear(e, looking_gear.held, true, true);
    }

    // --- Next Inventory Slot (Preview) ---
    if let Some(&e) = player_gear.inventory.first() {
        // Icon is visible in the stowed preview, and sound is audible
        process_gear(e, false, true, true);
    }

    // --- Deployed Gear ---
    // Check all deployed gear with minimal visibility requirements.
    for (entity, gear_pos, _deployed) in q_deployed_gear.iter() {
        let dist = player_pos.distance(gear_pos);
        // Approx 10 tiles ~ 15-20 meters.
        if dist < 12.0 {
            // Visible nearby. Status text is small, readable if very close.
            let can_read_status = dist < 2.5;
            // Sound is audible if nearby.
            let can_hear = dist < 12.0;
            // Icon is the gear sprite itself.
            let can_see_icon = dist < 12.0;

            process_gear(entity, can_read_status, can_see_icon, can_hear);
        }
    }

    // --- Special Non-Gear Evidence ---

    // UV Ectoplasm: Requires Ghost to be lit by UV Light.
    // RL Presence: Requires Ghost to be lit by Red Light.

    // Iterate over ghosts to see if they are illuminated
    for (ghost_sprite, ghost_pos) in q_ghost.iter() {
        if ghost_sprite.hunting > 0.0 {
            // ghost is hunting.
        }
        // Check lighting on ghost
        // Simple proximity check to active light sources
        for (light, toggle, light_pos) in q_light.iter() {
            if !toggle.is_on {
                continue;
            }
            let range = 6.0; // Approx effective range for flashlight beam intensity
            if light_pos.distance(ghost_pos) < range {
                match light.light_type {
                    LightType::UltraViolet => {
                        // UV Evidence
                        if haunt_state.evidences.contains(&Evidence::UVEctoplasm) {
                            evidence_readings.report_clarity(
                                Evidence::UVEctoplasm,
                                1.0,
                                current_time,
                                delta_time,
                            );
                        }
                    }
                    LightType::Red => {
                        // RL Presence Evidence
                        if haunt_state.evidences.contains(&Evidence::RLPresence) {
                            evidence_readings.report_clarity(
                                Evidence::RLPresence,
                                1.0,
                                current_time,
                                delta_time,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_current_evidence_readings_from_player_perception_system,
    );
}
