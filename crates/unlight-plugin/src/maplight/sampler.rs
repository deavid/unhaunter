use crate::maplight::definitions::FlashlightData;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use ndarray::Array3;
use std::cell::RefCell;
use unboard_core::resources::board_topology::BoardTopology;
use unfoundation_core::types::light::LightType;
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::tonemapping::{self, TonemappingParams};
use unlight_core::types::light::{LightData, LightFieldData};
use unrender_std::components::visuals::{LightSensitive, SpectralInfluence, SpectralInfluenceType};
use unrender_std::resources::visibility_data::VisibilityData;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;
use untypes_core::difficulty::Difficulty;

pub(crate) fn calculate_tutorial_light_factor(difficulty: &Difficulty) -> f32 {
    match difficulty {
        Difficulty::TutorialChapter1 => 1.0,
        Difficulty::TutorialChapter2 => 0.8,
        Difficulty::TutorialChapter3 => 0.6,
        Difficulty::TutorialChapter4 => 0.4,
        Difficulty::TutorialChapter5 => 0.2,
        _ => 0.0,
    }
}

pub(crate) struct LightingSampler<'a> {
    pub(crate) flashlights: &'a [FlashlightData],
    pub(crate) bf: &'a BoardTopology,
    pub(crate) lf: &'a Array3<LightFieldData>,
    pub(crate) vf: &'a VisibilityData,
    pub(crate) exposure: f32,
    pub(crate) tutorial_light_factor: f32,
    pub(crate) cache_tiles: RefCell<HashMap<BoardPosition, ((f32, f32, f32), LightData)>>,
    pub(crate) cache_corners: RefCell<HashMap<BoardPosition, (f32, f32, Color, LightData)>>,
    pub(crate) tonemap: TonemappingParams,
}

pub(crate) struct SpectralParams {
    pub(crate) att_charge: f32,
    pub(crate) rep_charge: f32,
    pub(crate) uv_charge: f32,
    pub(crate) red_charge: f32,
    pub(crate) ir_charge: f32,
}

impl From<Option<&SpectralInfluence>> for SpectralParams {
    fn from(si: Option<&SpectralInfluence>) -> Self {
        si.map(|x| {
            let (att, rep) = match x.influence_type {
                SpectralInfluenceType::Attractive => (x.charge_value.abs().sqrt() + 0.01, 0.0),
                SpectralInfluenceType::Repulsive => (0.0, x.charge_value.abs().sqrt() + 0.01),
            };
            Self {
                att_charge: att,
                rep_charge: rep,
                uv_charge: x.uv_charge,
                red_charge: x.red_charge,
                ir_charge: x.ir_charge,
            }
        })
        .unwrap_or(Self {
            att_charge: 0.0,
            rep_charge: 0.0,
            uv_charge: 0.0,
            red_charge: 0.0,
            ir_charge: 0.0,
        })
    }
}

impl<'a> LightingSampler<'a> {
    pub(crate) fn new(
        flashlights: &'a [FlashlightData],
        bf: &'a BoardTopology,
        lg: &'a LightGrid,
        vf: &'a VisibilityData,
        difficulty: &Difficulty,
    ) -> Self {
        let tutorial_light_factor = calculate_tutorial_light_factor(difficulty);
        let tonemap = TonemappingParams::new(
            tutorial_light_factor,
            tonemapping::DARK_COLOR2,
            tonemapping::BRIGHTNESS,
            lg.exposure.current,
        );
        Self {
            flashlights,
            bf,
            lf: &lg.light_field,
            vf,
            exposure: lg.exposure.current,
            tutorial_light_factor,
            cache_tiles: RefCell::new(HashMap::new()),
            cache_corners: RefCell::new(HashMap::new()),
            tonemap,
        }
    }

    pub(crate) fn apply_spectral_modulation(
        &self,
        r: &mut f32,
        g: &mut f32,
        b: &mut f32,
        ld: &LightData,
        sp: &SpectralParams,
    ) -> f32 {
        let rgbl = (*r + *g + *b) / 3.0 + 1.0;

        // Combine instant light with persistent charge
        let u = ld.ultraviolet.max(sp.uv_charge);
        let i = ld.infrared.max(sp.ir_charge);
        let rd = ld.red.max(sp.red_charge);

        *g += u * sp.att_charge * 2.5 * rgbl;
        *b += i * (sp.att_charge + sp.rep_charge) * 2.5 * rgbl;
        *b += rd * sp.rep_charge * 0.01 * rgbl;
        *r /= 1.0 + rd * sp.rep_charge * 50.0 * rgbl + u * sp.att_charge * 12.0 * rgbl;
        *g /= 1.0 + rd * sp.rep_charge * 10.0 * rgbl;
        *b /= 1.0
            + i * (sp.att_charge + sp.rep_charge) * 10.0 * rgbl
            + u * sp.att_charge * 12.0 * rgbl;
        (*r + *g + *b) / 3.0
    }

    pub(crate) fn fpos_gamma_color(
        &self,
        target_pos: Position,
        is_light_sensitive: bool,
    ) -> Option<((f32, f32, f32), LightData)> {
        let rpos_raw = target_pos;
        let bpos = target_pos.to_board_position();
        let p = bpos.ndidx_checked(self.bf.map_size)?;

        let is_integer_tile = !is_light_sensitive
            && (rpos_raw.x - bpos.x as f32).abs() < 0.01
            && (rpos_raw.y - bpos.y as f32).abs() < 0.01;

        if is_integer_tile && let Some(res) = self.cache_tiles.borrow().get(&bpos) {
            return Some(*res);
        }

        let mut lux_fl = [0_f32; 3];
        let mut lightdata = LightData::default();
        for flash in self.flashlights.iter() {
            let flvis = flash.vis_field[p];
            if flvis < 0.000001 {
                continue;
            }

            let flcolor = &flash.color;
            let fltype = &flash.light_type;

            let d2 = rpos_raw.distance2(&flash.pos);
            let fl = if is_light_sensitive && d2 < 4.0 {
                // Smooth, distance-only lighting for players/ghosts near light sources
                // Using a 1/r falloff for proximity boost as requested.
                let dist = d2.sqrt();
                // flpower is already adjusted. The 2.0 factor provides a damped proximity boost.
                flash.power / (dist + 5.0) * 2.0
            } else {
                let focus = flash.beam_focus;
                let lpos_unrot = flash.lpos_unrot;
                let mut rpos = Position {
                    x: rpos_raw.x * flash.unrot_axes[0].dx
                        + rpos_raw.y * flash.unrot_axes[1].dx
                        + rpos_raw.z * flash.unrot_axes[2].dx,
                    y: rpos_raw.x * flash.unrot_axes[0].dy
                        + rpos_raw.y * flash.unrot_axes[1].dy
                        + rpos_raw.z * flash.unrot_axes[2].dy,
                    z: rpos_raw.x * flash.unrot_axes[0].dz
                        + rpos_raw.y * flash.unrot_axes[1].dz
                        + rpos_raw.z * flash.unrot_axes[2].dz,
                    visual_priority: rpos_raw.visual_priority,
                };

                rpos.x -= lpos_unrot.x;
                rpos.y -= lpos_unrot.y;

                // Built-in softness and minimum width
                const MIN_SPREAD: f32 = 2.5;

                // 1. Bright Hotspot Logic
                let mut spot_rpos = rpos;
                if spot_rpos.x >= 0.0 {
                    spot_rpos.x = fastapprox::faster::pow(spot_rpos.x, 1.0 / focus.clamp(1.0, 1.3));
                    spot_rpos.y /= spot_rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + MIN_SPREAD;
                } else {
                    spot_rpos.x =
                        -fastapprox::faster::pow(-spot_rpos.x, (focus / 5.0 + 1.0).clamp(1.0, 4.0));
                    spot_rpos.y *= -spot_rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                    spot_rpos.y /= MIN_SPREAD;
                }

                let emitter_pos = Position {
                    x: 0.0,
                    y: 0.0,
                    z: lpos_unrot.z,
                    visual_priority: 0.0,
                };

                let dist = (emitter_pos.distance(&spot_rpos) + 0.5)
                    .powf((flash.dir.distance() / 200.0).clamp(0.5, 1.0).recip());
                let spot_fl = flash.power_f / (dist + 0.5);

                // 2. Faint Cone / Spotlight Beam Logic
                let beam_len = flash.dir.distance() / 30.0;
                let x_player_rel = rpos.x + beam_len;
                let mut cone_fl = 0.0;
                if x_player_rel > 0.0 && rpos.x < 0.0 {
                    let progress = (x_player_rel / beam_len).clamp(0.0, 1.0);

                    // Dynamic narrowing: narrower angle as we aim further (150-400 magnitude)
                    let range_factor = ((flash.dir.distance() - 150.0) / 250.0).clamp(0.0, 1.0);
                    let angle_multiplier = 1.0 - (range_factor * 0.6); // Up to 60% narrower at max range

                    // Cone width scales with distance, but is suppressed at long range to focus the beam
                    let cone_width = (progress * 15.0 + 5.0) * angle_multiplier;

                    let dz = rpos.z - lpos_unrot.z;
                    let lateral_dist = (rpos.y * rpos.y + dz * dz).sqrt();
                    let angular_falloff =
                        (1.0 - (lateral_dist / cone_width)).clamp(0.0, 1.0).powi(2);

                    // Intensity is shifted to the "later parts" (closer to hotspot)
                    // Peaks around 66% of the way to the hotspot and fades quickly at the player
                    let intensity_mod = progress.powi(2) * (1.0 - progress) * 6.75;
                    cone_fl = flash.power_f * 0.20 * intensity_mod * angular_falloff;
                }

                (spot_fl + cone_fl) * flvis.clamp(0.0001, 1.0)
            };
            let flsrgba = flcolor.to_srgba();
            lux_fl[0] += fl * flsrgba.red;
            lux_fl[1] += fl * flsrgba.green;
            lux_fl[2] += fl * flsrgba.blue;
            let ld = LightData::from_type(*fltype, fl);
            lightdata = lightdata.add(&ld);
        }
        let ambient_light = 0.0001 + self.tutorial_light_factor * 0.0001;
        let res = self.lf.get(bpos.ndidx()).map(|lf| {
            let r = (lf.lux * lf.color.0 + lux_fl[0] + ambient_light) / self.exposure;
            let g = (lf.lux * lf.color.1 + lux_fl[1] + ambient_light) / self.exposure;
            let b = (lf.lux * lf.color.2 + lux_fl[2] + ambient_light) / self.exposure;

            (
                (
                    tonemapping::artistic_tonemap(r, self.exposure),
                    tonemapping::artistic_tonemap(g, self.exposure),
                    tonemapping::artistic_tonemap(b, self.exposure),
                ),
                lightdata.add(&LightData::from_type(
                    LightType::Visible,
                    lf.lux + ambient_light,
                )),
            )
        });

        if is_integer_tile && let Some(r) = res {
            self.cache_tiles.borrow_mut().insert(bpos, r);
        }
        res
    }

    pub(crate) fn f_vis(&self, v: f32) -> f32 {
        (v.clamp(0.0, 1.0) * 1.5).clamp(0.0001, 1.0)
    }

    pub(crate) fn fpos_sampling_corner(
        &self,
        target_pos: Position,
        o_light_sens: Option<&LightSensitive>,
    ) -> (f32, f32, Color, LightData) {
        let x = target_pos.x;
        let y = target_pos.y;
        let z = target_pos.z.round() as i64;
        let x0 = x.floor() as i64;
        let y0 = y.floor() as i64;
        let x1 = x.ceil() as i64;
        let y1 = y.ceil() as i64;

        let is_light_sensitive = o_light_sens.is_some();

        // Corner cache for non-sensitive grid-aligned corners
        let bpos_corner = target_pos.to_board_position();
        let is_grid_corner = !is_light_sensitive
            && (target_pos.x * 2.0).fract() == 0.0
            && (target_pos.y * 2.0).fract() == 0.0;

        if is_grid_corner && let Some(res) = self.cache_corners.borrow().get(&bpos_corner) {
            return *res;
        }

        let mut total_l = 0.0;
        let mut total_v = 0.0;
        let mut total_r = 0.0;
        let mut total_g = 0.0;
        let mut total_b = 0.0;
        let mut total_ld = LightData::default();
        let mut count = 0.0;
        for tx in [x0, x1] {
            for ty in [y0, y1] {
                let bpos = BoardPosition { x: tx, y: ty, z };
                if let Some(p) = bpos.ndidx_checked(self.bf.map_size) {
                    let vis = self.f_vis(self.vf.visibility_field[p]);
                    if vis > 0.0001
                        && let Some(((r, g, b), ld)) =
                            self.fpos_gamma_color(bpos.to_position(), is_light_sensitive)
                    {
                        total_r += r;
                        total_g += g;
                        total_b += b;
                        total_l += (r + g + b) / 3.0;
                        total_v += vis;
                        total_ld = total_ld.add(&ld);
                        count += 1.0;
                    }
                }
            }
        }
        let res = if count > 0.0 {
            (
                total_l / count,
                (total_v / count).clamp(0.0001, 1.0),
                Color::srgb(total_r / count, total_g / count, total_b / count),
                total_ld.scale(1.0 / count),
            )
        } else {
            let vis = self.f_vis(
                self.vf
                    .visibility_field
                    .get(target_pos.to_board_position().ndidx())
                    .copied()
                    .unwrap_or(0.0),
            );
            let ((r, g, b), ld) = self
                .fpos_gamma_color(target_pos, is_light_sensitive)
                .unwrap_or(((1.0, 1.0, 1.0), LightData::UNIT_VISIBLE));
            ((r + g + b) / 3.0, vis, Color::srgb(r, g, b), ld)
        };
        if is_grid_corner {
            self.cache_corners.borrow_mut().insert(bpos_corner, res);
        }
        res
    }

    pub(crate) fn calc_gamma(&self, lux: f32, tc: f32) -> f32 {
        tonemapping::calc_gamma(lux, tc, &self.tonemap)
    }

    pub(crate) fn calc_rgba(
        &self,
        lux: f32,
        visibility: f32,
        bcolor: Option<Color>,
        dst_color: Color,
        is_tile: bool,
        map_color_alpha: f32,
    ) -> LinearRgba {
        tonemapping::calc_rgba(
            lux,
            visibility,
            bcolor,
            dst_color,
            is_tile,
            map_color_alpha,
            &self.tonemap,
        )
    }
}
