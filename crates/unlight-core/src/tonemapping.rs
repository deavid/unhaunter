use bevy::prelude::*;

pub const DARK_COLOR2: Color = Color::srgba(0.2, 0.6, 1.0, 1.0);
pub const BRIGHTNESS: f32 = 2.5;

/// Blue tint threshold: perceived brightness below this gets progressively blue-tinted.
const BLUE_TINT_THRESHOLD: f32 = 0.5;

/// Artistic tonemapping: Compression curve modulated by eye adaptation.
///
/// - When dark-adapted (low exposure): High contrast, fast saturation.
///   Small lights look bright (clipping), darkness is lifted.
/// - When bright-adapted (high exposure): Gentle compression, high dynamic range.
/// - Bloom: When a light is much brighter than the adapted level (x > 3),
///   output exceeds 1.0, producing washout/blinding in the shader.
pub fn artistic_tonemap(x: f32, exposure: f32) -> f32 {
    // When exposure is low (e.g. 0.1), k drops to ~0.4 (steep curve).
    // When exposure is high (e.g. 1.0), k rises to ~0.75 (gentle S-curve).
    let k = (exposure * 0.75).clamp(0.40, 2.0);
    let compressed = x.powf(1.5) / (x.powf(1.5) + k);
    // Bloom: when over-exposed (x >> adapted level), allow output > 1.0
    // for blinding/washout effect.
    let bloom = (x - 1.0).max(0.0) * 0.5;
    compressed + bloom
}

pub struct TonemappingParams {
    pub tutorial_light_factor: f32,
    pub dark_color2: Vec4,
    pub brightness: f32,
    pub exposure: f32,
}

impl TonemappingParams {
    pub fn new(
        tutorial_light_factor: f32,
        dark_color2: Color,
        brightness: f32,
        exposure: f32,
    ) -> Self {
        Self {
            tutorial_light_factor,
            dark_color2: LinearRgba::from(dark_color2).to_vec4(),
            brightness,
            exposure,
        }
    }
}

/// Compute the blue tint factor based on absolute lux.
fn blue_tint_factor(absolute_lux: f32, params: &TonemappingParams) -> f32 {
    let base = ((BLUE_TINT_THRESHOLD - absolute_lux) / BLUE_TINT_THRESHOLD).clamp(0.0, 1.0);
    (base + params.tutorial_light_factor * 0.2).clamp(0.0, 1.0)
}

/// Compute per-vertex gamma (brightness) for the shader.
///
/// `lux` is the already-tonemapped perceived brightness.
/// `tc` is tint_comp: darkness of the texture's own color
/// (0.0 = bright texture, 1.0 = dark texture).
pub fn calc_gamma(lux: f32, tc: f32, params: &TonemappingParams) -> f32 {
    // lux arrives already tonemapped via artistic_tonemap in the sampler.
    // Do NOT tonemap again — double-sigmoid breaks monotonicity.
    let tm_lux = lux.max(0.0);

    // Base gamma directly from tonemapped brightness
    let base_gamma = tm_lux * params.brightness;

    // Small boost for dark textures so they don't vanish completely
    let tc_boost = tc * 0.05 / (1.0 + tm_lux * 4.0);

    // Compensate gamma for blue tint darkening (blue tint reduces R/G channels)
    let absolute_lux = lux * params.exposure;
    let blue_t = blue_tint_factor(absolute_lux, params);
    let blue_comp = 1.0 + blue_t * 0.05;

    ((base_gamma + tc_boost) * blue_comp).max(0.001)
}

/// Compute per-vertex color tint for the shader.
///
/// `lux` is the already-tonemapped perceived brightness.
/// Applies blue night-vision tint in dark areas, neutral/white in bright areas.
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
    // Blue tint based on absolute lux
    let absolute_lux = lux * params.exposure;
    let blue_t = blue_tint_factor(absolute_lux, params);
    let white_v = Vec4::ONE;
    let tint_v = white_v + (params.dark_color2 - white_v) * blue_t;

    let mut base_v = LinearRgba::from(dst_color).to_vec4();
    if is_tile {
        if let Some(bc) = bcolor {
            let bc_srgba = bc.to_srgba();
            let max_c = bc_srgba.red.max(bc_srgba.green).max(bc_srgba.blue).max(0.2);
            base_v = Vec4::new(
                bc_srgba.red / max_c,
                bc_srgba.green / max_c,
                bc_srgba.blue / max_c,
                base_v.w,
            );
        }
        // Alpha from visibility
        base_v.w = (visibility * map_color_alpha).clamp(0.0, 1.0);
    }
    // For non-tile, rgba.w already comes from dst_color which includes spectral/manual alpha

    let rgba_v = base_v * tint_v;
    LinearRgba::from_vec4(rgba_v)
}
