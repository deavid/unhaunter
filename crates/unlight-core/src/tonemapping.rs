use bevy::prelude::*;

pub const K_COLD: f32 = 0.6;
pub const DARK_COLOR2: Color = Color::srgba(0.2, 0.6, 1.0, 1.0);
pub const BRIGHTNESS: f32 = 1.0;

/// Artistic tonemapping: Sigmoid-ish curve with highlight protection (shoulder).
pub fn artistic_tonemap(x: f32) -> f32 {
    let v = x.powf(1.3);
    v * 1.8 / (1.0 + v * 0.45)
}

pub fn f_gamma(lux: f32) -> f32 {
    fastapprox::faster::pow(lux, 0.9)
}

pub fn calc_gamma(lux: f32, tc: f32, exposure: f32, k_cold: f32, brightness: f32) -> f32 {
    let p_cold_f = (1.0 - (lux / k_cold).tanh()) * 2.0;
    let p_exp_color =
        ((-(exposure + 0.0001).ln() / 2.0 - 1.5 + p_cold_f).tanh() + 0.5).clamp(0.0, 1.0);

    // Ensure lux has a tiny floor to prevent precision divergence in pitch black
    let lux_f = lux.max(0.0001);

    f_gamma(
        lux_f * brightness * (1.0 + p_cold_f + (p_exp_color * 2.0).powi(2))
            + (tc + p_cold_f * 2.0 + (p_exp_color * 2.0).powi(2)) / (10.0 + exposure + lux_f),
    ) + p_exp_color / 40.0
}

#[allow(clippy::too_many_arguments)]
pub fn calc_rgba(
    lux: f32,
    visibility: f32,
    bcolor: Option<Color>,
    exposure: f32,
    k_cold: f32,
    tutorial_light_factor: f32,
    dark_color2: Color,
    dst_color: Color,
    is_tile: bool,
    map_color_alpha: f32,
) -> LinearRgba {
    let p_cold_f = (1.0 - (lux / k_cold).tanh()) * 2.0;
    let p_exp_color =
        ((-(exposure + 0.0001).ln() / 2.0 - 1.5 + p_cold_f).tanh() + 0.5).clamp(0.0, 1.0);
    let p_exp_color_tint = (p_exp_color + tutorial_light_factor * 0.20).clamp(0.0, 1.0);

    // Use a clamped lux for the dark color lerp to ensure consistency in pitch black
    let color_lux = lux.max(0.001);

    // Inline lerp_color logic to avoid dependency on unrender-std
    let start = Color::WHITE.to_srgba();
    let end = dark_color2.to_srgba();
    let t = p_exp_color_tint / f_gamma(color_lux).clamp(1.0, 300.0);
    let p_dark2 = Color::srgba(
        start.red + (end.red - start.red) * t,
        start.green + (end.green - start.green) * t,
        start.blue + (end.blue - start.blue) * t,
        start.alpha + (end.alpha - start.alpha) * t,
    );

    let mut base = dst_color;
    if is_tile && let Some(bc) = bcolor {
        let bc_srgba = bc.to_srgba();
        let max_c = bc_srgba.red.max(bc_srgba.green).max(bc_srgba.blue).max(0.2);
        base = Color::srgb(
            bc_srgba.red / max_c,
            bc_srgba.green / max_c,
            bc_srgba.blue / max_c,
        );
    }

    let mut rgba = LinearRgba::from(base).to_vec4() * LinearRgba::from(p_dark2).to_vec4();

    // Scale RGB if it's a tile
    if is_tile {
        let tmp_val = LinearRgba::from_vec4(rgba);
        let lum = tmp_val.luminance();
        let target_lum = 0.8;
        let intensity_factor = target_lum / lum.max(0.01);
        rgba.x *= intensity_factor;
        rgba.y *= intensity_factor;
        rgba.z *= intensity_factor;

        // Apply anti-pitch-black ambient logic per-corner to ensure smoothness
        const DARK_COLOR_CORNER: Color = Color::srgba(0.247 / 1.5, 0.714 / 1.5, 0.878, 1.0);
        let dark_f = (p_exp_color_tint / 16.0 + tutorial_light_factor * 0.01).clamp(0.0, 1.0);
        let dark_spike = LinearRgba::from(DARK_COLOR_CORNER).to_vec4() * dark_f;

        rgba.x += dark_spike.x;
        rgba.y += dark_spike.y;
        rgba.z += dark_spike.z;

        // Alpha from visibility (smooth for tiles)
        rgba.w = (visibility * map_color_alpha).clamp(0.0, 1.0);
    }
    // For non-tile, rgba.w already comes from dst_color which includes spectral/manual alpha

    LinearRgba::from_vec4(rgba)
}
