use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_platform::collections::{HashMap, HashSet};
use rand::RngExt;

use unbehavior_core::behavior::Behavior;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::MapTileSprite;
use unboard_core::resources::visibility_data::VisibilityData;
use uncommon_app_core::random_seed;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfog_core::components::MiasmaSprite;
use ungear_core::components::playergear::PlayerGear;
use unghost_core::components::presentation::spectral::SpectralClarity;
use uninput_core::resources::MouseVisibility;
use uninteraction_core::hover::HoverState;
use unlight_core::color_utils::lerp_color;
use unlight_core::components::LightSensitive;
use unlight_core::flashlight::ActiveFlashlights;
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::spectral::SpectralInfluence;
use unlight_core::types::light::LightData;
use unplayer_core::components::MainPlayer;
use unrender_std::components::visuals::{AlphaModulator, EctoplasmVisuals, Emissive, Ethereal};
use unrender_std::custom_material1::CustomMaterial1;
use unreplicon_core::ownership::LocallyOwned;
use unsettings_core::video::VideoSettings;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::direction::Direction;
use unspatial_core::orientation::Orientation;
use unspatial_core::perspective;
use unspatial_core::position::Position;

use crate::definitions::GridResources;
use crate::metrics::APPLY_LIGHTING;
use crate::sampler::{LightingSampler, SpectralParams};
use crate::visuals::{
    apply_alpha_modulator_visuals, apply_ecto_visuals, apply_emissive_visuals,
    apply_ethereal_visuals, apply_ir_visuals, apply_miasma_pressure, apply_uv_visuals,
    step_alpha_clamped, update_spectral_influence,
};
use unmetrics_core::metrics::SendMetric;

#[expect(clippy::type_complexity)]
pub(crate) fn apply_lighting_to_tiles_system(
    mut qt2: Query<
        (
            Entity,
            &Position,
            &MeshMaterial2d<CustomMaterial1>,
            &mut Visibility,
            Option<&Behavior>,
            Option<&mut SpectralInfluence>,
            Option<&HoverState>,
            Option<&Ethereal>,
            Option<&EctoplasmVisuals>,
            Option<&SpectralClarity>,
            (
                Option<&LightSensitive>,
                Option<&MapColor>,
                Option<&MiasmaSprite>,
                Option<&AlphaModulator>,
                Option<&Emissive>,
            ),
            Has<LocallyOwned>,
        ),
        With<MapTileSprite>,
    >,
    materials1: ResMut<Assets<CustomMaterial1>>,
    qp: Query<(&Position, &Direction, &PlayerGear, Has<MainPlayer>)>,
    q_special: Query<
        Entity,
        (
            With<MapTileSprite>,
            Or<(
                With<Ethereal>,
                With<EctoplasmVisuals>,
                With<AlphaModulator>,
                With<HoverState>,
                With<LightSensitive>,
            )>,
        ),
    >,
    active_flashlights: Res<ActiveFlashlights>,
    mut lg: If<ResMut<LightGrid>>,
    grids: GridResources,
    // haunt_state: Res<HauntState>,
    q_vf: Query<&VisibilityData, With<MainPlayer>>,
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    mut visible: Local<HashSet<Entity>>,
    mouse_visibility: Res<MouseVisibility>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    let bf = &grids.bf;
    let bef = &grids.bef;
    let miasma = &grids.miasma;
    let video_quality = video_settings.quality.to_quality_factor3();

    let measure = APPLY_LIGHTING.time_measure();

    let Ok(vf) = q_vf.single() else {
        return;
    };

    let mut rng = random_seed::rng();

    // --- High-Level Intent: Perceptual Scene Reconstruction ---
    // The goal of this function is to transform a mathematical simulation of photons
    // into an atmospheric, readable visual scene that reacts to the player's presence.

    let mut player_pos = Position::new_i64(0, 0, 0);
    let elapsed = time.elapsed_secs();

    if bf.map_size.0 == 0 {
        // If we don't have a valid map, skip this
        return;
    }

    // Check if visibility field is properly initialized
    if vf.visibility_field.is_empty() {
        return;
    }

    if let Some((pos, _direction, _gear, _)) = qp.iter().find(|x| x.3) {
        player_pos = *pos;
    }

    let mut lightdata_map: HashMap<BoardPosition, LightData> = HashMap::new();

    // Primes: 13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,173,281,409,541,659,809
    const VSMALL_PRIME: usize = 59;
    const BIG_PRIME: usize = 95629;
    let mask: usize = rng.random_range(0..usize::MAX);

    // --- Shared Lighting Sampling Logic ---

    let sampler = LightingSampler::new(&active_flashlights.list, bf, &lg, vf, &difficulty.0);

    // --- End of Shared Lighting Sampling Logic ---

    // let start = Instant::now();
    let materials1 = materials1.into_inner();

    let update_radius: usize = rng.random_range(8..32);
    let player_bpos = player_pos.to_board_position();
    let (map_width, map_height, map_depth) = bf.map_size;
    let player_ndidx = player_bpos.ndidx();
    let min_x = (player_ndidx.0).saturating_sub(update_radius);
    let max_x = (player_ndidx.0 + update_radius).min(map_width - 1);
    let min_y = (player_ndidx.1).saturating_sub(update_radius);
    let max_y = (player_ndidx.1 + update_radius).min(map_height - 1);
    let min_z = player_ndidx.2.saturating_sub(1);
    let max_z = (player_ndidx.2 + 1).min(map_depth - 1);
    let mut entities = Vec::with_capacity(256);

    let video_quality_recip = video_quality.recip().round() as usize;

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let n = x + y * map_width + z * map_width * map_height;
                let dist = ((player_ndidx.0 as isize - x as isize).abs()
                    + (player_ndidx.1 as isize - y as isize).abs()
                    + (player_ndidx.2 as isize - z as isize).abs())
                    as usize
                    + 1;
                let min_threshold = ((n * BIG_PRIME) ^ mask) % VSMALL_PRIME;
                let skip_tile = min_threshold * dist * video_quality_recip / 9
                    > update_radius.saturating_sub(dist + 2);
                if vf.visibility_field[(x, y, z)] > 0.00001 {
                    if skip_tile {
                        // Skip
                    } else {
                        entities.extend_from_slice(&bef.0[(x, y, z)]);
                    }
                }
            }
        }
    }

    // --- Special Entity Updates (Always) ---
    // Entities that are interactive, ethereal, etc. must update every frame
    // to ensure smooth animation and responsiveness.
    for entity in q_special.iter() {
        if visible.contains(&entity) {
            entities.push(entity);
        }
    }

    // --- Flashlight Coverage Updates ---
    // Tiles touched by active flashlights must update to ensure crisp edges.
    // Iterating dense field might be faster than spatial query if N is small.
    for fl in active_flashlights.list.iter() {
        // Optimization: We could use bounding box, but iterating array is simd-fast.
        // We only check if > 0.
        // To speed this up, we can use the flashlight pos and direction to
        // limit the iteration range.
        let c_z = fl.pos.z as isize;

        // Fallback to full simple box for now to rely on vis_field array access speed
        // Actually, iterating 64x64 is tiny (4096).
        // Let's just iterate the whole valid Z plane for the flashlight
        let fl_z = c_z.clamp(0, map_depth as isize - 1) as usize;

        // We use the pre-computed vis_field to just pick what is lit.
        // Note: vis_field is 3D.
        // Iterating only relevant Z slices.
        for z in fl_z.saturating_sub(1)..=fl_z.min(map_depth - 1) {
            // Only iterate a loose bounding box around the player/flashlight
            // Because flashlights can be 200m long, we should arguably iterate the whole map
            // or trust the pre-computed bounds.
            // For now, let's just iterate the sub-volume likely to be affected.
            // A 64x64 loop is cheap enough.
            for x in 0..map_width {
                for y in 0..map_height {
                    if fl.vis_field[(x, y, z)] > 0.001 {
                        entities.extend_from_slice(&bef.0[(x, y, z)]);
                    }
                }
            }
        }
    }

    // --- Background Decay (Random Sample) ---
    // Iterate visible set to find candidates for "turning off".
    // We do NOT check components here to avoid O(N) query lookup overhead.
    // Purely random sampling.
    let decay_rate = 5; // 5% per frame -> ~0.3s to full sweep
    for e in visible.iter() {
        if rng.random_range(0..100) < decay_rate {
            entities.push(*e);
        }
    }

    // Sort and Deduplicate
    entities.sort_unstable();
    entities.dedup();

    for entity in entities.iter() {
        let min_threshold: f32 = rng.random::<f32>() / 10.0 / video_quality;
        if let Ok((
            _entity_id,
            pos,
            mat,
            mut vis,
            o_behavior,
            mut o_spectral_influence,
            o_hover,
            o_ethereal,
            o_ecto_vis,
            o_spectral_clarity,
            (o_light_sens, o_map_color, o_miasma, o_alpha_mod, o_emissive),
            is_locally_owned,
        )) = qt2.get_mut(*entity)
        {
            if !is_locally_owned {
                // Temporary debug log for remote entities
                // warn!("Processing remote entity: {:?}", _entity_id);
            }
            // --- Per-Entity Spectral & Visual Processing ---
            // This is the core 'flavor' of the investigation mechanics.
            // Objects react differently to UV, Red, and IR light, sometimes 'charging'
            // or glowing based on their paranormal properties.
            let on_hover = o_hover
                .map(|x| x.is_hovered && mouse_visibility.is_visible)
                .unwrap_or_default();
            let mut opacity: f32 = 1.0;

            let mut bpos = pos.to_board_position_size(bf.map_size);
            if let Some(behavior) = o_behavior {
                bpos.x += behavior.p.display.light_recv_offset.0;
                bpos.y += behavior.p.display.light_recv_offset.1;
                if behavior.p.display.auto_hide {
                    // Make big objects semitransparent when the player is behind them
                    const MAX_DIST: f32 = 8.0;
                    let dist = pos.distance(&player_pos);
                    if dist < MAX_DIST {
                        let delta_z = perspective::to_screen_coord(*pos).z
                            - perspective::to_screen_coord(player_pos).z;
                        if delta_z > 0.0 {
                            opacity = 0.1;
                        }
                    }
                }
            } else {
                let visibility: f32 = vf.visibility_field[bpos.ndidx()].clamp(0.0, 1.0);
                opacity = (visibility * 1.5).clamp(0.0001, 1.0);
            }

            // Use a margin (that should be baked on the map) to avoid negative access.
            if bpos.x < 2 || bpos.y < 2 {
                continue;
            }

            let mut lux_c;
            let vis_c = sampler.f_vis(vf.visibility_field[bpos.ndidx()]);

            // Corner offsets correspond to tile intersections.
            // In our isometric perspective (PERSPECTIVE_X/Y in perspective.rs):
            // (+0.5, +0.5) -> Screen Right
            // (-0.5, -0.5) -> Screen Left
            // (-0.5, +0.5) -> Screen Top
            // (+0.5, -0.5) -> Screen Bottom
            let (mut lux_right, vis_right, mut color_right, ld_right) = sampler
                .fpos_sampling_corner(
                    *pos + Direction {
                        dx: 0.5,
                        dy: 0.5,
                        dz: 0.0,
                    },
                    o_light_sens,
                );
            let (mut lux_left, vis_left, mut color_left, ld_left) = sampler.fpos_sampling_corner(
                *pos + Direction {
                    dx: -0.5,
                    dy: -0.5,
                    dz: 0.0,
                },
                o_light_sens,
            );
            let (mut lux_top, vis_top, mut color_top, ld_top) = sampler.fpos_sampling_corner(
                *pos + Direction {
                    dx: -0.5,
                    dy: 0.5,
                    dz: 0.0,
                },
                o_light_sens,
            );
            let (mut lux_bot, vis_bot, mut color_bot, ld_bot) = sampler.fpos_sampling_corner(
                *pos + Direction {
                    dx: 0.5,
                    dy: -0.5,
                    dz: 0.0,
                },
                o_light_sens,
            );

            let ((mut r, mut g, mut b), _raw, light_data) = sampler
                .fpos_gamma_color(*pos, o_light_sens.is_some())
                .unwrap_or(((1.0, 1.0, 1.0), (1.0, 1.0, 1.0), LightData::UNIT_VISIBLE));
            if let Some(ls) = o_light_sens {
                r = (r + ls.bias).max(0.05);
                g = (g + ls.bias).max(0.05);
                b = (b + ls.bias).max(0.05);
            }

            let dt = time.delta_secs();

            // Update spectral charges if the entity has SpectralInfluence
            if let Some(ref mut si) = o_spectral_influence {
                update_spectral_influence(si, &light_data, dt);
            }
            let sp = SpectralParams::from(o_spectral_influence.as_deref());

            let res = sampler.apply_spectral_modulation(&mut r, &mut g, &mut b, &light_data, &sp);
            lux_c = (res / 10.0).tanh() * 10.0;

            let process_corner = |lux: &mut f32, color: &mut Color, ld: &LightData| {
                let srgba = color.to_srgba();
                let (mut cr, mut cg, mut cb) = (srgba.red, srgba.green, srgba.blue);
                *lux = sampler.apply_spectral_modulation(&mut cr, &mut cg, &mut cb, ld, &sp);
                *color = Color::srgb(cr, cg, cb);
            };

            process_corner(&mut lux_right, &mut color_right, &ld_right);
            process_corner(&mut lux_left, &mut color_left, &ld_left);
            process_corner(&mut lux_top, &mut color_top, &ld_top);
            process_corner(&mut lux_bot, &mut color_bot, &ld_bot);

            let ld = light_data.normalize();

            if let Some(behavior) = o_behavior
                && behavior.p.movement.walkable
            {
                lightdata_map.insert(bpos.clone(), light_data);
            }
            let max_color = r.max(g).max(b).max(0.2);
            let mut src_color_base = Color::srgb(r / max_color, g / max_color, b / max_color);
            let mut smooth_f: f32;
            let mut smooth_a: f32 = 1.0;

            let is_tile = o_behavior.is_some();
            if is_tile {
                // Tiles DO NOT need instant synchronization to avoid seams between neighbors. It is fine.
                smooth_f = 0.5;
            } else {
                // Players and characters need smoother color transitions than tiles
                smooth_f = 3.0;
            }
            let map_color = o_map_color
                .map(|x| x.color)
                .unwrap_or(Color::LinearRgba(LinearRgba::rgb(1.0, 1.0, 1.0)));

            if let Some(ethereal) = o_ethereal {
                smooth_f = 299.0;
                smooth_a = 199.0;
                if ethereal.warning_active {
                    src_color_base = lerp_color(
                        css::RED.into(),
                        css::ALICE_BLUE.into(),
                        ethereal.warning_intensity.clamp(0.0, 1.0),
                    );
                } else if ethereal.hunt_target {
                    src_color_base = lerp_color(
                        css::RED.into(),
                        css::ALICE_BLUE.into(),
                        (ethereal.calm_time_secs / 10.0).clamp(0.0, 1.0),
                    );
                }
            }

            if let Some(_ecto) = o_ecto_vis {
                smooth_f = 99.0;
                smooth_a = 99.0;
            }

            let occlusion = o_behavior
                .map(|b| b.obsolete_occlusion_type())
                .unwrap_or(Orientation::None);
            match occlusion {
                Orientation::None => {}
                Orientation::XAxis => {
                    lux_top = lux_c;
                    lux_bot = lux_c;
                }
                Orientation::YAxis => {
                    lux_right = lux_c;
                    lux_left = lux_c;
                }
                Orientation::Both => {
                    lux_top = lux_c;
                    lux_bot = lux_c;
                    lux_right = lux_c;
                    lux_left = lux_c;
                }
            }
            opacity = opacity
                .min(vf.visibility_field[bpos.ndidx()] * 2.0)
                .clamp(0.0, 1.0);
            let mut new_mat = materials1.get(&mat.0).unwrap().clone();
            let orig_mat = new_mat.clone();

            // remove brightness calculation for main tile:
            let mut dst_color = src_color_base;

            let difficulty_val = &difficulty.0;

            if let Some(am) = o_alpha_mod {
                apply_alpha_modulator_visuals(am, elapsed, &mut opacity);
            }

            if let Some(ethereal) = o_ethereal.filter(|e| !e.warning_active && !e.hunt_target) {
                apply_ethereal_visuals(
                    ethereal,
                    o_spectral_clarity,
                    &ld,
                    difficulty_val,
                    elapsed,
                    &mut opacity,
                    &mut dst_color,
                );
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

            if let Some(_miasma_sprite) = o_miasma {
                apply_miasma_pressure(pos, miasma, bf, &mut opacity);
            }

            if let Some(emissive) = o_emissive {
                apply_emissive_visuals(
                    emissive,
                    &light_data,
                    elapsed,
                    vf.visibility_field[bpos.ndidx()],
                    &mut dst_color,
                );
            }

            if let Some(si) = o_spectral_influence.as_deref() {
                apply_uv_visuals(
                    si,
                    &ld,
                    vf.visibility_field[bpos.ndidx()],
                    &mut opacity,
                    &mut dst_color,
                );
                apply_ir_visuals(
                    si,
                    &light_data,
                    vf.visibility_field[bpos.ndidx()],
                    &mut opacity,
                );
            }

            if !is_tile {
                let mc = map_color.to_linear().to_vec4();
                let sc = dst_color.to_linear().to_vec4();
                dst_color = LinearRgba::from_vec4(mc * sc * 0.9 + mc * 0.1).into();
            }
            let prev_a = new_mat.data.color.alpha();
            dst_color.set_alpha(step_alpha_clamped(
                opacity,
                prev_a,
                smooth_a,
                o_behavior.is_none(),
                map_color.alpha(),
            ));

            new_mat.data.ambient_color = Color::NONE.into();

            let tint_comp = (1.0 - src_color_base.luminance()).clamp(0.0, 1.0);
            let smooth_f = prev_a + 0.3 + smooth_f;

            let gamma_mean = |a: f32, b: f32, tc: f32| {
                (a * smooth_f + sampler.calc_gamma(b, tc)) / (1.0 + smooth_f)
            };

            let color_mean = |a: LinearRgba, b: LinearRgba| -> LinearRgba {
                let a_v = a.to_vec4();
                let b_v = b.to_vec4();
                LinearRgba::from_vec4((a_v * smooth_f + b_v) / (1.0 + smooth_f))
            };

            // Mapping to vertices:
            // gtl (Top): Logic (0.5, -0.5)
            // gtr (Right): Logic (0.5, 0.5)
            // gbl (Left): Logic (-0.5, -0.5)
            // gbr (Bottom): Logic (-0.5, 0.5)

            // For tiles, we use a neutral tint_comp for corners to ensure neighbors agree on the value.
            let corner_tc = if is_tile { 0.5 } else { tint_comp };

            new_mat.data.gamma = gamma_mean(new_mat.data.gamma, lux_c, tint_comp);
            if is_tile {
                new_mat.data.gtl = gamma_mean(new_mat.data.gtl, lux_top, corner_tc);
                new_mat.data.gtr = gamma_mean(new_mat.data.gtr, lux_right, corner_tc);
                new_mat.data.gbl = gamma_mean(new_mat.data.gbl, lux_left, corner_tc);
                new_mat.data.gbr = gamma_mean(new_mat.data.gbr, lux_bot, corner_tc);

                let new_c_c =
                    sampler.calc_rgba(lux_c, vis_c, None, dst_color, is_tile, map_color.alpha());
                let new_c_tl = sampler.calc_rgba(
                    lux_top,
                    vis_top,
                    Some(color_top),
                    dst_color,
                    is_tile,
                    map_color.alpha(),
                );
                let new_c_tr = sampler.calc_rgba(
                    lux_right,
                    vis_right,
                    Some(color_right),
                    dst_color,
                    is_tile,
                    map_color.alpha(),
                );
                let new_c_bl = sampler.calc_rgba(
                    lux_left,
                    vis_left,
                    Some(color_left),
                    dst_color,
                    is_tile,
                    map_color.alpha(),
                );
                let new_c_br = sampler.calc_rgba(
                    lux_bot,
                    vis_bot,
                    Some(color_bot),
                    dst_color,
                    is_tile,
                    map_color.alpha(),
                );

                new_mat.data.color = color_mean(new_mat.data.color, new_c_c);
                new_mat.data.ctl = color_mean(new_mat.data.ctl, new_c_tl);
                new_mat.data.ctr = color_mean(new_mat.data.ctr, new_c_tr);
                new_mat.data.cbl = color_mean(new_mat.data.cbl, new_c_bl);
                new_mat.data.cbr = color_mean(new_mat.data.cbr, new_c_br);
            } else {
                new_mat.data.gtl = new_mat.data.gamma;
                new_mat.data.gtr = new_mat.data.gamma;
                new_mat.data.gbl = new_mat.data.gamma;
                new_mat.data.gbr = new_mat.data.gamma;

                let new_c =
                    sampler.calc_rgba(lux_c, vis_c, None, dst_color, is_tile, map_color.alpha());

                new_mat.data.color = color_mean(new_mat.data.color, new_c);
                new_mat.data.color.alpha = new_c.alpha;
                new_mat.data.ctl = new_mat.data.color;
                new_mat.data.ctr = new_mat.data.color;
                new_mat.data.cbl = new_mat.data.color;
                new_mat.data.cbr = new_mat.data.color;
            }

            if on_hover {
                lux_c += 1.0;
                new_mat.data.ambient_color = Color::srgb(0.20, 0.20, 0.0).into();

                let mut h_color = new_mat.data.color.to_vec4();
                h_color.x = (h_color.x + 0.5).min(1.0);
                h_color.y = (h_color.y + 0.5).min(1.0);
                h_color.z *= 0.3;
                new_mat.data.color = LinearRgba::from_vec4(h_color);
                if !is_tile {
                    new_mat.data.ctl = new_mat.data.color;
                    new_mat.data.ctr = new_mat.data.color;
                    new_mat.data.cbl = new_mat.data.color;
                    new_mat.data.cbr = new_mat.data.color;
                }

                // We update gamma with the hover boost too
                new_mat.data.gamma = gamma_mean(new_mat.data.gamma, lux_c, tint_comp);
                if !is_tile {
                    new_mat.data.gtl = new_mat.data.gamma;
                    new_mat.data.gtr = new_mat.data.gamma;
                    new_mat.data.gbl = new_mat.data.gamma;
                    new_mat.data.gbr = new_mat.data.gamma;
                }
            }

            let invisible = (is_tile && new_mat.data.color.alpha() < 0.005)
                || o_behavior.map(|b| b.p.display.disable).unwrap_or_default();
            let new_vis = if invisible {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            if new_vis != *vis {
                if invisible {
                    visible.remove(entity);
                } else {
                    visible.insert(entity.to_owned());
                }
                *vis = new_vis;
            } else if !invisible && !visible.contains(entity) {
                visible.insert(entity.to_owned());
            }
            let delta = orig_mat.data.delta(&new_mat.data);
            let thr = 0.02;
            let auto_hide = o_behavior
                .map(|b| b.p.display.auto_hide)
                .unwrap_or_default();
            if auto_hide
                || delta > thr + min_threshold
                || o_ethereal.is_some()
                || o_ecto_vis.is_some()
                || o_behavior.is_none()
            {
                let mat = materials1.get_mut(&mat.0).unwrap();
                mat.data = new_mat.data;
                // change_count += 1;
            }
        }
    }

    for (bpos, ld) in lightdata_map.into_iter() {
        lg.light_field[bpos.ndidx()].additional = ld;
    }

    measure.end_ms();
}
