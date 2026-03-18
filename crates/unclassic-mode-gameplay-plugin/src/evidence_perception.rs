use bevy::prelude::*;
use unlight_core::types::LightType;
use ungear_core::components::core::{Electronic, EvidenceSensor, PerceivedClarity};
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::looking_gear::LookingGear;
use unghost_core::components::ghost_orb_particle::GhostOrbParticle;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use unghost_core::types::evidence::Evidence;
use uninteraction_core::interaction::Toggleable;
use unlight_core::resources::light_grid::LightGrid;
use unrender_std::components::light::LightEmitter;
use unrender_std::resources::visibility_data::VisibilityData;
use unspatial_core::position::Position;
use untags_core::tags::PlayerTag;

fn update_current_evidence_readings_from_player_perception_system(
    mut evidence_readings: ResMut<CurrentEvidenceReadings>,
    player_query: Query<(&PlayerGear, &Position, Option<&VisibilityData>), With<PlayerTag>>,
    looking_gear: If<Res<LookingGear>>,
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
    light_grid: If<Res<LightGrid>>,
    time: Res<Time>,
) {
    let Some((player_gear, player_pos, vis_data)) = player_query.iter().next() else {
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

        // Strict Floor Check: Gear on a different floor (Z-level) is ignored.
        // Assuming integer Z levels, a difference of >= 0.5 means a different floor.
        if (player_pos.z - gear_pos.z).abs() >= 0.5 {
            continue;
        }

        // Approx 10 tiles ~ 15-20 meters.
        // The player needs to be somewhat close to validly "perceive" the evidence.
        if dist < 8.0 {
            let mut is_visible = true;
            // Check visibility against obstructions (walls)
            if let Some(vis) = vis_data {
                let bpos = gear_pos.to_board_position();
                let dims = vis.visibility_field.dim();
                if let Some(v_idx) = bpos.ndidx_checked(dims) {
                    if let Some(v) = vis.visibility_field.get(v_idx) {
                        // Visibility is 0.0 to 1.0. Typically < 0.0 is uninitialized, 0.0 is occlusion.
                        if *v <= 0.0 {
                            is_visible = false;
                        }
                    } else {
                        is_visible = false;
                    }
                } else {
                    // Out of bounds
                    is_visible = false;
                }
            }

            // Visible nearby. Status text is small, readable if very close and visible.
            let can_read_status = is_visible && dist < 2.5;
            // Sound is audible if nearby. Reduced to require closer proximity.
            // Sound flows through walls to some extent, but we count it only if somewhat close.
            // But if it's completely invisible (thick wall), maybe we should suppress?
            // For now, let's keep sound independent of visibility but rely on distance and floor check.
            let can_hear = dist < 6.5;
            // Icon is the gear sprite itself. Reduced significantly so player must be close to "see" the reading.
            let can_see_icon = is_visible && dist < 4.5;

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
        let ghost_evidences = ghost_sprite.class.evidences();
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
                        if ghost_evidences.contains(&Evidence::UVEctoplasm) {
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
                        if ghost_evidences.contains(&Evidence::RLPresence) {
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
