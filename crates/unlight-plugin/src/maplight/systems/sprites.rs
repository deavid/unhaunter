use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use rand::Rng;
use unboard_core::components::mapcolor::MapColor;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfog_core::components::MiasmaSprite;
use unfoundation_core::random_seed;
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::components::salt::UVReactive;
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::types::light::LightData;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::MainPlayer;
use unrender_std::components::game::MapTileSprite;
use unrender_std::components::visuals::{
    AlphaModulator, EctoplasmVisuals, Ethereal, InfraredSensitive, LightSensitive, ShadowCaster,
    SpectralClarity, SpectralInfluence, UltravioletSensitive,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::light::lerp_color;
use unsettings_core::video::VideoSettings;
use unspatial_core::position::Position;

use crate::maplight::definitions::{ActiveFlashlights, GridResources};
use crate::maplight::sampler::{LightingSampler, SpectralParams};
use crate::maplight::visuals::{
    apply_alpha_modulator_visuals, apply_ecto_visuals, apply_ethereal_visuals, apply_ir_visuals,
    apply_miasma_cloud_visuals, apply_uv_visuals, update_spectral_influence,
};
use crate::metrics;

pub(crate) fn highlight_placement_tiles_system(
    qp: Query<(&Position, &PlayerGear, Has<MainPlayer>)>,
    mut tile_sprites: Query<
        (&Position, &mut Sprite),
        (
            With<MapTileSprite>,
            Without<Ethereal>,
            Without<ShadowCaster>,
        ),
    >,
) {
    let Some((p_pos, player_gear, _)) = qp.iter().find(|x| x.2) else {
        return;
    };
    if player_gear.held_item.is_none() {
        return;
    }

    // Only highlight if the player is holding an object
    let target_tile = p_pos.to_board_position();
    for (tile_pos, mut sprite) in tile_sprites.iter_mut() {
        if tile_pos.to_board_position() == target_tile {
            // Adjust highlight color and intensity as needed
            let highlight_color = Color::srgba(0.0, 1.0, 0.0, 0.3);
            sprite.color = lerp_color(sprite.color, highlight_color, 0.5);
        }
    }
}

/// System to apply lighting and visual effects to all game sprites.
#[expect(clippy::type_complexity)]
pub(crate) fn apply_lighting_to_sprites_system(
    mut qt: Query<
        (
            Entity,
            &Position,
            Option<&mut Sprite>,
            Option<&MeshMaterial2d<CustomMaterial1>>,
            Option<&mut SpectralInfluence>,
            Option<&Ethereal>,
            Option<&EctoplasmVisuals>,
            Option<&SpectralClarity>,
            (
                Option<&LightSensitive>,
                Option<&InfraredSensitive>,
                Option<&UltravioletSensitive>,
                Option<&ShadowCaster>,
                Option<&MapColor>,
                Option<&UVReactive>,
                Option<&MiasmaSprite>,
                Option<&AlphaModulator>,
            ),
        ),
        Without<MapTileSprite>,
    >,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    active_flashlights: Res<ActiveFlashlights>,
    lg: Res<LightGrid>,
    grids: GridResources,
    q_vf: Query<&VisibilityData, With<MainPlayer>>,
    difficulty: Res<CurrentDifficulty>,
    time: Res<Time>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    let Ok(vf) = q_vf.single() else {
        return;
    };
    let measure = metrics::APPLY_LIGHTING_SPRITES.time_measure();
    let bf = &grids.bf;
    let miasma = &grids.miasma;
    let miasma_config = &grids.miasma_config;
    let quality_factor = video_settings.quality.to_quality_factor3();
    let elapsed = time.elapsed_secs();
    let dt = time.delta_secs();

    let sampler = LightingSampler::new(
        &active_flashlights.list,
        bf,
        &lg,
        vf,
        &difficulty.0.difficulty,
    );

    let mut rng = random_seed::rng();

    for (
        _entity,
        pos,
        o_sprite,
        o_mat,
        mut o_spectral_influence,
        o_ethereal,
        o_ecto_vis,
        o_spectral_clarity,
        (
            o_light_sens,
            o_ir_sens,
            o_uv_sens,
            o_shadow_caster,
            o_color,
            uv_reactive,
            o_miasma,
            o_alpha_mod,
        ),
    ) in qt.iter_mut()
    {
        let sprite_color = if let Some(sprite) = o_sprite.as_ref() {
            sprite.color
        } else if let Some(mat_handle) = o_mat {
            materials1
                .get(&mat_handle.0)
                .map(|m| Color::from(m.data.color))
                .unwrap_or(Color::WHITE)
        } else {
            continue;
        };
        // Reduce the chances of refreshing the sprite color by the quality setting
        if rng.random_range(0.0..2.0) > quality_factor {
            continue;
        }
        let bpos = pos.to_board_position_size(bf.map_size);
        let map_color = o_color.map(|x| x.color).unwrap_or_default();
        let visibility: f32 = vf.visibility_field[bpos.ndidx()].clamp(0.0, 1.0);
        let mut opacity: f32 = map_color.alpha() * visibility;
        opacity = (opacity.powf(0.5) * 2.0 - 0.1).clamp(0.0001, 1.0);

        let ((mut r, mut g, mut b), ld_abs) = sampler
            .fpos_gamma_color(*pos, o_light_sens.is_some())
            .unwrap_or(((1.0, 1.0, 1.0), LightData::UNIT_VISIBLE));

        if let Some(light_sens) = o_light_sens {
            r = (r + light_sens.bias).max(0.05);
            g = (g + light_sens.bias).max(0.05);
            b = (b + light_sens.bias).max(0.05);
        }

        // Update spectral charges if the entity has SpectralInfluence
        if let Some(ref mut si) = o_spectral_influence {
            update_spectral_influence(si, &ld_abs, dt);
        }
        let sp = SpectralParams::from(o_spectral_influence.as_deref());

        sampler.apply_spectral_modulation(&mut r, &mut g, &mut b, &ld_abs, &sp);

        let ld = ld_abs.normalize();

        let mut dst_color = Color::srgb(r, g, b);
        let mut smooth_a: f32 = 1.0;

        if let Some(uv_sens) = o_uv_sens {
            apply_uv_visuals(uv_sens, &ld, &mut dst_color, &mut opacity);
        }

        if let Some(uv_react) = uv_reactive {
            let uv_react = uv_react.0;
            dst_color = lerp_color(
                dst_color,
                css::GREEN.into(),
                (ld.ultraviolet * uv_react).sqrt(),
            );
            // Multiplier for fluorescence
            let mut srgba = dst_color.to_srgba();
            let l = (srgba.red + srgba.green + srgba.blue) / 3.0;
            let boost = ld.ultraviolet * uv_react * 5.0;
            srgba.red += boost * l;
            srgba.green += boost * l;
            srgba.blue += boost * l;
            dst_color = srgba.into();
        }

        if let Some(ir_sens) = o_ir_sens {
            apply_ir_visuals(ir_sens, &ld_abs, visibility, &mut opacity, &mut smooth_a);
        }

        if let Some(am) = o_alpha_mod {
            apply_alpha_modulator_visuals(am, elapsed, &mut opacity);
        }

        let difficulty_val = &difficulty.0;

        if let Some(ethereal) = o_ethereal {
            if !ethereal.warning_active && !ethereal.hunt_target {
                apply_ethereal_visuals(
                    ethereal,
                    o_spectral_clarity,
                    &ld,
                    difficulty_val,
                    elapsed,
                    &mut opacity,
                    &mut dst_color,
                );
            } else {
                // Handle warning/hunt colors (copied from tile logic for now)
                dst_color = if ethereal.warning_active {
                    lerp_color(
                        css::RED.into(),
                        css::ALICE_BLUE.into(),
                        ethereal.warning_intensity.clamp(0.0, 1.0),
                    )
                } else {
                    lerp_color(
                        css::RED.into(),
                        css::ALICE_BLUE.into(),
                        (ethereal.calm_time_secs / 10.0).clamp(0.0, 1.0),
                    )
                };
            }
        }

        if let Some(_ecto) = o_ecto_vis.filter(|e| e.use_breach_curve) {
            apply_ecto_visuals(
                &ld,
                difficulty_val,
                elapsed,
                &mut opacity,
                &mut dst_color,
                vf.visibility_field[bpos.ndidx()],
                &mut rng,
            );
        }

        if let Some(miasma_sprite) = o_miasma {
            apply_miasma_cloud_visuals(
                miasma_sprite,
                pos,
                &ld,
                miasma,
                miasma_config,
                quality_factor,
                &mut dst_color,
                &mut opacity,
            );
        }

        let old_a = (sprite_color.alpha()).clamp(0.0001, 1.0);
        dst_color.set_alpha(
            ((opacity + old_a * smooth_a) / (smooth_a + 1.0)).clamp(0.0, 1.0) * map_color.alpha(),
        );

        let src_linear = sprite_color.to_linear();
        let dst_linear = dst_color.to_linear();
        let f = if o_shadow_caster.is_some() {
            0.01
        } else {
            0.11
        };
        let smooth_color = LinearRgba::from_vec4(
            (src_linear.to_vec4() * (1.0 - f) + dst_linear.to_vec4() * f)
                .clamp(Vec4::ZERO, Vec4::ONE),
        );
        if let Some(mut sprite) = o_sprite {
            sprite.color = smooth_color.into();
        } else if let Some(mat_handle) = o_mat
            && let Some(mat) = materials1.get_mut(&mat_handle.0)
        {
            mat.data.color = smooth_color;
        }
    }
    measure.end_ms();
}
