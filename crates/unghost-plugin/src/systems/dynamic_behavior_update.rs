use bevy::prelude::*;
use unboard_core::components::physics::{FluidEmitter, ThermalEmitter};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::haunt_state::HauntState;
use uninvestigation_core::evidence::Evidence;
use unmetrics_core::metrics::SendMetric;
use unnoise_core::perlin::{LONG_TERM_NOISE_FREQ, PerlinNoise, SHORT_TERM_NOISE_FREQ};
use unsoundfield_core::components::SoundFieldSource;

use crate::metrics;

/// Helper function to calculate a noise-based multiplier value
///
/// This function combines short-term and long-term noise values with given offsets,
/// normalizes them, combines them, applies power scaling, and clamps the result.
fn calculate_noise_multiplier(
    noise_table: &PerlinNoise,
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
    combined_noise.powf(power_scale) * 2.0 - 1.0 // Scale to [-1, 1]
}

fn update_ghost_behavior_dynamics_system(
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    noise_table: Res<PerlinNoise>,
    mut query: Query<(&GhostSprite, &mut GhostBehaviorDynamics)>,
    mut report_time: Local<f32>,
) {
    let measure = metrics::GHOST_BEHAVIOR_DYNAMICS.time_measure();
    let elapsed_seconds = time.elapsed_secs();
    let evidence_visibility_recip = difficulty.0.evidence_visibility().recip();
    *report_time += time.delta_secs();
    for (ghost_sprite, mut dynamics) in query.iter_mut() {
        // Iterate through all 8 actual Evidence enum variants
        for evidence_type in Evidence::all() {
            // Evidence::all() comes from enum_iterator trait
            let (offset_x, offset_y) = dynamics.noise_offsets.get_evidence_offsets(evidence_type);
            // Some evidences are easier than others, so this is to adjust and make it fair.
            // Lower values = evidence is more stable/easier to detect
            let ev_diff_mult = match evidence_type {
                Evidence::FreezingTemp => 0.1,
                Evidence::FloatingOrbs => 1.0,
                Evidence::UVEctoplasm => 1.0,
                Evidence::EMFLevel5 => 1.0,
                Evidence::EVPRecording => 1.0,
                Evidence::SpiritBox => 1.0,
                Evidence::RLPresence => 1.0,
                Evidence::CPM500 => 0.1,
            };
            let scaled_value = calculate_noise_multiplier(
                &noise_table,
                elapsed_seconds,
                offset_x,
                offset_y,
                evidence_visibility_recip * ev_diff_mult,
            );

            let evidence_presence_max = if ghost_sprite.class.evidences().contains(&evidence_type) {
                1.0
            } else {
                0.0
            };

            let final_clarity_value = scaled_value.clamp(-1.0, evidence_presence_max);
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
            debug!(
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
    measure.end_ms();
}

fn sync_ghost_field_sources(
    haunt_state: Res<HauntState>,
    mut q_emitters: Query<(
        &mut ThermalEmitter,
        &mut FluidEmitter,
        &mut SoundFieldSource,
        Option<&GhostSprite>,
        Option<&GhostBreach>,
    )>,
) {
    let measure = metrics::GHOST_EMITTER_SYNC.time_measure();
    let freezing = haunt_state.ghost_dynamics.freezing_temp_clarity;
    let ghost_target_temp =
        uncommon_app_core::utils::temperature::celsius_to_kelvin(1.0 - 4.0 * freezing);
    let power = freezing * 0.5 + 0.5;

    const GHOST_MAX_POWER: f32 = 0.01;
    const BREACH_MAX_POWER: f32 = 10.0;

    for (mut thermal, mut fluid, mut sound, opt_ghost, opt_breach) in q_emitters.iter_mut() {
        if opt_ghost.is_none() && opt_breach.is_none() {
            continue;
        }
        thermal.target_temp = ghost_target_temp;
        if opt_ghost.is_some() {
            thermal.power = GHOST_MAX_POWER * power;
        } else {
            // It's the breach
            thermal.power = BREACH_MAX_POWER * power;
        }

        // Fluid emitter logic (based on existence currently)
        fluid.pressure = 1.0;

        // Sound emitter logic (volume)
        sound.volume = 1.0;
    }
    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut bevy::prelude::App) {
    app.add_systems(
        bevy::prelude::Update,
        (
            update_ghost_behavior_dynamics_system
                .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
            sync_ghost_field_sources,
        )
            .chain(),
    );
}
