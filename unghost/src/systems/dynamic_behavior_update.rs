use bevy::prelude::*;
use uncore::{
    components::{ghost_behavior_dynamics::GhostBehaviorDynamics, ghost_sprite::GhostSprite},
    difficulty::CurrentDifficulty,
    types::evidence::Evidence,
};

use uncore::noise::{LONG_TERM_NOISE_FREQ, SHORT_TERM_NOISE_FREQ};

/// Helper function to calculate a noise-based multiplier value
///
/// This function combines short-term and long-term noise values with given offsets,
/// normalizes them, combines them, applies power scaling, and clamps the result.
fn calculate_noise_multiplier(
    noise_table: &uncore::noise::PerlinNoise,
    elapsed_seconds: f32,
    offset_x: f32,
    offset_y: f32,
    power_scale: f32,
) -> f32 {
    let short_term_noise = noise_table.get(
        elapsed_seconds * SHORT_TERM_NOISE_FREQ + offset_x,
        elapsed_seconds * LONG_TERM_NOISE_FREQ + offset_y,
    );
    let long_term_noise = noise_table.get(
        elapsed_seconds * LONG_TERM_NOISE_FREQ + offset_x * -1.5,
        elapsed_seconds * LONG_TERM_NOISE_FREQ * 0.1 + offset_y * 3.3,
    );
    let sum = (short_term_noise + long_term_noise) * 2.0;

    let combined_noise = sum.tanh() * 0.5 + 0.5;
    let scaled_value = combined_noise.powf(power_scale);

    scaled_value.clamp(0.0, 1.0)
}

pub(crate) fn update_ghost_behavior_dynamics_system(
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    noise_table: Res<uncore::noise::PerlinNoise>,
    mut query: Query<(&GhostSprite, &mut GhostBehaviorDynamics)>,
    mut report_time: Local<f32>,
) {
    let elapsed_seconds = time.elapsed_secs();
    let evidence_visibility_recip = difficulty.0.evidence_visibility.recip();
    *report_time += time.delta_secs();
    for (ghost_sprite, mut dynamics) in query.iter_mut() {
        // Iterate through all 8 actual Evidence enum variants
        for evidence_type in Evidence::all() {
            // Evidence::all() comes from enum_iterator trait
            let (offset_x, offset_y) = dynamics.noise_offsets.get_evidence_offsets(evidence_type);

            let scaled_value = calculate_noise_multiplier(
                &noise_table,
                elapsed_seconds,
                offset_x,
                offset_y,
                evidence_visibility_recip,
            );

            let evidence_presence_multiplier =
                if ghost_sprite.class.evidences().contains(&evidence_type) {
                    1.0
                } else {
                    0.0
                };

            let final_clarity_value = scaled_value * evidence_presence_multiplier;
            dynamics.set_clarity(evidence_type, final_clarity_value);
        }

        // Update visual_alpha_multiplier
        dynamics.visual_alpha_multiplier = calculate_noise_multiplier(
            &noise_table,
            elapsed_seconds,
            dynamics.noise_offsets.visual_alpha_multiplier_x,
            dynamics.noise_offsets.visual_alpha_multiplier_y,
            evidence_visibility_recip,
        );

        // Update rage_tendency_multiplier
        dynamics.rage_tendency_multiplier = calculate_noise_multiplier(
            &noise_table,
            elapsed_seconds,
            dynamics.noise_offsets.rage_tendency_multiplier_x,
            dynamics.noise_offsets.rage_tendency_multiplier_y,
            evidence_visibility_recip,
        );
        if *report_time > 10.0 {
            info!(
                "Dynamics: Frz:{:.2}, Orbs:{:.2}, UV:{:.2}, EMF:{:.2}, EVP:{:.2}, SprtBx:{:.2}, RL:{:.2}, CPM500:{:.2}, Alpha:{:.2}, Rage:{:.2}",
                dynamics.freezing_temp_clarity,
                dynamics.floating_orbs_clarity,
                dynamics.uv_ectoplasm_clarity,
                dynamics.emf_level5_clarity,
                dynamics.evp_recording_clarity,
                dynamics.spirit_box_clarity,
                dynamics.rl_presence_clarity,
                dynamics.cpm500_clarity,
                dynamics.visual_alpha_multiplier,
                dynamics.rage_tendency_multiplier
            );
            *report_time = 0.0;
        }
    }
}
