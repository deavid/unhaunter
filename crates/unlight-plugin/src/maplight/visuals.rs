use bevy::color::palettes::css;
use bevy::prelude::*;
use rand::Rng;
use rand::RngExt;
use unboard_core::resources::board_topology::BoardTopology;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unfog_core::components::MiasmaSprite;
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;
use unlight_core::types::light::LightData;
use unrender_std::components::visuals::{
    AlphaModulator, Emissive, Ethereal, InfraredSensitive, SpectralClarity, UltravioletSensitive,
};
use unrender_std::utils::light::lerp_color;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

pub(crate) fn update_spectral_influence(
    si: &mut unrender_std::components::visuals::SpectralInfluence,
    light_data: &LightData,
    dt: f32,
) {
    let update_charge = |target: f32, current: &mut f32| {
        let delta = target - *current;
        let tau = if delta > 0.0 { 1.0 } else { 0.25 };
        let alpha = 1.0 - (-dt / tau).exp();
        *current += delta * alpha;
    };
    update_charge(light_data.ultraviolet, &mut si.uv_charge);
    update_charge(light_data.red, &mut si.red_charge);
    update_charge(light_data.infrared, &mut si.ir_charge);
}

pub(crate) fn apply_miasma_pressure(
    pos: &Position,
    miasma: &MiasmaGrid,
    bf: &BoardTopology,
    opacity: &mut f32,
) {
    let mut total_pressure = 0.0_f32;
    let bpos_m = pos.to_board_position();
    for npos in bpos_m.iter_xy_neighbors(1, bf.map_size) {
        total_pressure += miasma.pressure_field[npos.ndidx()];
    }
    total_pressure /= 10.0;
    *opacity *= (1.0_f32 - total_pressure).clamp(0.0_f32, 1.0_f32);
}

pub(crate) fn apply_uv_visuals(
    uv_sens: &UltravioletSensitive,
    ld: &LightData,
    visibility: f32,
    dst_color: &mut Color,
    opacity: &mut f32,
) {
    *opacity = (*opacity + ld.ultraviolet * uv_sens.intensity * visibility).clamp(0.0, 1.3);
    let f = (ld.ultraviolet * uv_sens.color_shift * visibility).clamp(0.0, 1.0);
    *dst_color = lerp_color(*dst_color, css::MEDIUM_SLATE_BLUE.into(), f);
}

pub(crate) fn apply_ir_visuals(
    ir_sens: &InfraredSensitive,
    ld_abs: &LightData,
    visibility: f32,
    opacity: &mut f32,
    smooth: &mut f32,
) {
    if let Some(threshold) = ir_sens.thresholds {
        *smooth = 10.0;
        let total_light = ld_abs.visible + ld_abs.red + ld_abs.ultraviolet + ld_abs.infrared + 0.1;
        let ir_ratio = ld_abs.infrared / total_light;
        if ir_ratio > threshold && ld_abs.infrared > 0.1 && ld_abs.visible < 0.5 {
            *opacity = (ir_ratio * 2.0 - 1.0).powi(2) * ld_abs.infrared.sqrt() * visibility;
            *opacity = (*opacity * ir_sens.intensity).clamp(0.0, 1.0);
        } else {
            *opacity = 0.0;
        }
    } else {
        *opacity = (*opacity + ld_abs.infrared * ir_sens.intensity).clamp(0.0, 1.3);
    }
}

pub(crate) fn apply_alpha_modulator_visuals(am: &AlphaModulator, elapsed: f32, opacity: &mut f32) {
    *opacity *= (am.frequency * elapsed).sin() * am.amplitude + (1.0 - am.amplitude);
}

pub(crate) fn apply_ethereal_visuals<D: DifficultySettings>(
    ethereal: &Ethereal,
    spectral_clarity: Option<&SpectralClarity>,
    ld: &LightData,
    difficulty: &D,
    elapsed: f32,
    opacity: &mut f32,
    dst_color: &mut Color,
) {
    let orig_opacity = *opacity;

    // Make the ghost oscilate to increase visibility:
    let osc1 = (elapsed * 1.0 * difficulty.evidence_visibility()).sin() * 0.25 + 0.75;
    let osc2 = (elapsed * 1.15 * difficulty.evidence_visibility()).cos() * 0.5 + 0.5;
    *opacity =
        opacity.min(osc1 + 0.2) / (1.0 + ethereal.warp / 5.0) * difficulty.evidence_visibility();
    let l = (dst_color.luminance() + osc2) / 2.0;
    *dst_color = dst_color.with_luminance(l);
    let r = dst_color.to_srgba().red;
    let g = dst_color.to_srgba().green;
    let clarity = spectral_clarity.cloned().unwrap_or_default();
    let e_uv = ld.ultraviolet * 13.0 * clarity.uv.max(0.0);
    let e_rl = (ld.red * 52.0 * clarity.rl.max(0.0)).clamp(0.0, 1.5);
    let e_infra = (ld.infrared * 1.1 * difficulty.evidence_visibility()).sqrt();
    let f =
        (ld.visible * difficulty.evidence_visibility() * 0.5 + ld.infrared * 4.0).clamp(0.001, 0.999);
    *opacity = *opacity * f + orig_opacity * (1.0 - f);
    *opacity *= (clarity.alpha * 0.5
        + 0.5
        + e_uv
        + e_rl
        + ld.ultraviolet * 2.0
        + ld.red * 10.0
        + ld.infrared)
        .clamp(difficulty.evidence_visibility() * 0.1, 1.0);
    let srgba = dst_color
        .with_luminance((l * ld.visible - ld.infrared - ethereal.hit_delta * 3.0).clamp(0.0, 1.0))
        .to_srgba();

    let k_hit = (ethereal.hit_delta + ethereal.miss_delta)
        .clamp(0.0, 1.0)
        .cbrt();
    *opacity = *opacity * (1.0 - k_hit) + orig_opacity.cbrt() * k_hit;

    let mut final_color = srgba
        .with_red(r * ld.visible + e_rl * 1.1 + ethereal.miss_delta / 2.0)
        .with_green(g * ld.visible + e_uv + e_rl + ethereal.miss_delta / 2.5);

    if ethereal.warning_active || ethereal.hunt_target {
        // Make the ghost bright red and pulsing during a hunt/warning
        let pulse = (elapsed * 8.0).sin() * 0.5 + 0.5;
        let base_intensity = if ethereal.hunt_target {
            1.0
        } else {
            ethereal.warning_intensity
        };
        let intensity = base_intensity.max(0.5) + pulse * 0.2;

        final_color = final_color.with_red((final_color.red + intensity).max(1.0));
        final_color = final_color.with_green(final_color.green * 0.2);
        final_color = final_color.with_blue(final_color.blue * 0.2);

        *opacity = opacity.max(0.9);
    }
    *dst_color = final_color.into();
    *dst_color = dst_color.with_luminance((dst_color.luminance() - e_infra / 2.0).clamp(0.0, 1.0));
}

pub(crate) fn apply_ecto_visuals<D: DifficultySettings>(
    ld: &LightData,
    difficulty: &D,
    elapsed: f32,
    opacity: &mut f32,
    dst_color: &mut Color,
    visibility_at_pos: f32,
    mut rng: impl Rng,
) {
    let e_nv = ld.ultraviolet.cbrt() / 10.0 * difficulty.evidence_visibility() - ld.infrared * 3.0
        + (difficulty.evidence_visibility() / 7.0 + ld.visible * difficulty.evidence_visibility()
            - ld.infrared)
            .powi(3);
    *opacity *= ((dst_color.luminance() / 2.0) + e_nv / 4.0).clamp(0.0, 0.9);
    *opacity = opacity.min(visibility_at_pos).cbrt();
    let l = dst_color.luminance();
    let rnd_f = rng.random_range(-1.0..1.0_f32).powi(3);
    // Make the breach oscilate to increase visibility:
    let osc1 = ((elapsed * 0.92).sin() * 10.0 + 8.0).tanh() * 0.5 + 0.5;

    *dst_color =
        dst_color.with_luminance(((l * (ld.visible - ld.infrared) + e_nv) * osc1).clamp(0.0, 0.99));
    let lin_dst_color = dst_color.to_linear();

    *dst_color = lin_dst_color
        .with_green(
            lin_dst_color.green
                + ld.ultraviolet * difficulty.evidence_visibility() * (1.3 - osc1 + rnd_f / 14.0),
        )
        .with_red(
            lin_dst_color.red
                + ld.ultraviolet
                    * difficulty.evidence_visibility()
                    * 1.2
                    * (1.4 - osc1 + rnd_f / 24.0),
        )
        .into();
}

pub(crate) fn apply_miasma_cloud_visuals(
    miasma_sprite: &MiasmaSprite,
    pos: &Position,
    ld: &LightData,
    miasma: &MiasmaGrid,
    miasma_config: &MiasmaConfig,
    quality_factor: f32,
    dst_color: &mut Color,
    opacity: &mut f32,
) {
    let bpos = pos.to_board_position();
    let mut total_pressure = 0.0;
    let mut total_weight = 0.0;

    for dx in -1..=1 {
        for dy in -1..=1 {
            let neighbor_pos = BoardPosition {
                x: bpos.x + dx,
                y: bpos.y + dy,
                z: bpos.z,
            };

            if let Some(neighbor_pressure) = miasma.pressure_field.get(neighbor_pos.ndidx()) {
                let neighbor_center = neighbor_pos.to_position_center();
                let distance = pos.distance(&neighbor_center);
                let weight = (distance + 0.1).recip();

                total_pressure += neighbor_pressure * weight;
                total_weight += weight;
            }
        }
    }

    let average_pressure = if total_weight > 0.0 {
        total_pressure / total_weight
    } else {
        0.0
    };

    let miasma_visibility = average_pressure.max(0.0).sqrt()
        * miasma_config.miasma_visibility_factor
        * miasma_sprite.life.clamp(0.0, 1.0)
        * (ld.magnitude().atan() / 1.1 + 0.4)
        * (1.0 + (1.0 - quality_factor.clamp(0.1, 1.0)) * 0.35);

    *dst_color =
        dst_color.with_luminance((dst_color.luminance().sqrt() * 0.9 + 0.01).clamp(0.0, 1.0));
    *opacity = opacity.max(0.0);
    *opacity *= miasma_visibility.clamp(0.0, 0.8)
        * miasma_sprite.visibility
        * (dst_color.luminance().sqrt() * 0.8 + 0.2);
}

pub(crate) fn step_alpha_clamped(
    opacity: f32,
    prev_a: f32,
    smooth_a: f32,
    _is_special: bool,
    map_alpha: f32,
) -> f32 {
    const A_DELTA: f32 = 0.01;
    let f_a = 1.0 / (1.0 + smooth_a);

    let next_a = opacity * f_a + prev_a * (1.0 - f_a);
    let new_a = if (next_a - opacity).abs() < A_DELTA {
        opacity
    } else {
        next_a - A_DELTA * (next_a - opacity).signum()
    };

    (new_a * map_alpha).clamp(0.0, 1.0)
}

pub(crate) fn apply_emissive_visuals(
    emissive: &Emissive,
    ld_abs: &LightData,
    elapsed: f32,
    visibility: f32,
    dst_color: &mut Color,
) {
    let mut boost = emissive.intensity;
    if emissive.pulse_speed > 0.0 {
        boost *= (elapsed * emissive.pulse_speed).sin() * 0.15 + 0.85;
    }
    // Phosphorescence/Fluorescence: Stimulus in, light out.
    boost += ld_abs.magnitude() * emissive.light_reactivity;

    // Apply visibility to the final boost.
    // If we can't see the tile, we can't see its emission.
    boost *= visibility;

    let mut dcl = dst_color.to_linear();
    let ecl = emissive.color.to_linear();

    dcl.red += ecl.red * boost;
    dcl.green += ecl.green * boost;
    dcl.blue += ecl.blue * boost;

    *dst_color = dcl.into();
}
