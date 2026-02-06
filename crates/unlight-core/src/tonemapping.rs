use bevy::prelude::*;

pub const K_COLD: f32 = 0.6;
pub const DARK_COLOR2: Color = Color::srgba(0.2, 0.6, 1.0, 1.0);
pub const BRIGHTNESS: f32 = 1.0;

/// Artistic tonemapping: Sigmoid-ish curve with highlight protection (shoulder).
pub fn artistic_tonemap(x: f32) -> f32 {
    let v = fastapprox::faster::pow(x, 1.3);
    v * 1.8 / (1.0 + v * 0.45)
}

pub fn f_gamma(lux: f32) -> f32 {
    fastapprox::faster::pow(lux, 0.9)
}

pub struct TonemappingParams {
    pub exp_term: f32,
    pub k_cold: f32,
    pub exposure: f32,
    pub tutorial_light_factor: f32,
    pub dark_color2: Vec4,
    pub brightness: f32,
}

impl TonemappingParams {
    pub fn new(
        exposure: f32,
        k_cold: f32,
        tutorial_light_factor: f32,
        dark_color2: Color,
        brightness: f32,
    ) -> Self {
        Self {
            exp_term: -(exposure + 0.0001).ln() / 2.0 - 1.5,
            k_cold,
            exposure,
            tutorial_light_factor,
            dark_color2: LinearRgba::from(dark_color2).to_vec4(),
            brightness,
        }
    }
}

pub fn calc_gamma(lux: f32, tc: f32, params: &TonemappingParams) -> f32 {
    let lux_f = lux.max(0.0001);
    let p_cold_f = (1.0 - fastapprox::faster::tanh(lux_f / params.k_cold)) * 2.0;
    let p_exp_color = (fastapprox::faster::tanh(params.exp_term + p_cold_f) + 0.5).clamp(0.0, 1.0);

    f_gamma(
        lux_f * params.brightness * (1.0 + p_cold_f + (p_exp_color * 2.0).powi(2))
            + (tc + p_cold_f * 2.0 + (p_exp_color * 2.0).powi(2))
                / (10.0 + params.exposure + lux_f),
    ) + p_exp_color / 40.0
}

#[allow(clippy::too_many_arguments)]
pub fn calc_rgba(
    lux: f32,
    visibility: f32,
    bcolor: Option<Color>,
    dst_color: Color,
    is_tile: bool,
    map_color_alpha: f32,
    params: &TonemappingParams,
) -> LinearRgba {
    let lux_f = lux.max(0.0001);
    let p_cold_f = (1.0 - fastapprox::faster::tanh(lux_f / params.k_cold)) * 2.0;
    let p_exp_color = (fastapprox::faster::tanh(params.exp_term + p_cold_f) + 0.5).clamp(0.0, 1.0);
    let p_exp_color_tint = (p_exp_color + params.tutorial_light_factor * 0.20).clamp(0.0, 1.0);

    // Use a clamped lux for the dark color lerp to ensure consistency in pitch black
    let color_lux = lux_f.max(0.001);

    let t = p_exp_color_tint / f_gamma(color_lux).clamp(1.0, 300.0);
    let white_v = Vec4::ONE;
    let p_dark2_v = white_v + (params.dark_color2 - white_v) * t;

    let mut base_v = LinearRgba::from(dst_color).to_vec4();
    if is_tile && let Some(bc) = bcolor {
        let bc_srgba = bc.to_srgba();
        let max_c = bc_srgba.red.max(bc_srgba.green).max(bc_srgba.blue).max(0.2);
        base_v = Vec4::new(
            bc_srgba.red / max_c,
            bc_srgba.green / max_c,
            bc_srgba.blue / max_c,
            base_v.w,
        );
    }

    let mut rgba_v = base_v * p_dark2_v;

    // Scale RGB if it's a tile
    if is_tile {
        let lum = LinearRgba::from_vec4(rgba_v).luminance();
        let target_lum = 0.8;
        let intensity_factor = target_lum / lum.max(0.01);
        rgba_v.x *= intensity_factor;
        rgba_v.y *= intensity_factor;
        rgba_v.z *= intensity_factor;

        // Apply anti-pitch-black ambient logic per-corner to ensure smoothness
        const DARK_COLOR_CORNER: Vec4 = Vec4::new(0.247 / 1.5, 0.714 / 1.5, 0.878, 1.0);
        let dark_f =
            (p_exp_color_tint / 16.0 + params.tutorial_light_factor * 0.01).clamp(0.0, 1.0);
        let dark_spike = DARK_COLOR_CORNER * dark_f;

        rgba_v.x += dark_spike.x;
        rgba_v.y += dark_spike.y;
        rgba_v.z += dark_spike.z;

        // Alpha from visibility (smooth for tiles)
        rgba_v.w = (visibility * map_color_alpha).clamp(0.0, 1.0);
    }
    // For non-tile, rgba.w already comes from dst_color which includes spectral/manual alpha

    LinearRgba::from_vec4(rgba_v)
}
