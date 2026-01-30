use crate::maplight::definitions::FlashlightData;
use bevy::prelude::*;
use ndarray::Array3;
use unboard_core::resources::board_topology::BoardTopology;
use unfoundation_core::types::light::LightType;
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::tonemapping;
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
        Self {
            flashlights,
            bf,
            lf: &lg.light_field,
            vf,
            exposure: lg.exposure.current,
            tutorial_light_factor,
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
        const FL_MIN_DST: f32 = 0.1;
        let rpos_raw = target_pos;
        let bpos = target_pos.to_board_position();
        let p = bpos.ndidx_checked(self.bf.map_size)?;
        let mut lux_fl = [0_f32; 3];
        let mut lightdata = LightData::default();
        for flash in self.flashlights.iter() {
            let flpos = &flash.pos;
            let fldir = &flash.dir;
            let flpower = flash.power;
            let flcolor = &flash.color;
            let fltype = &flash.light_type;
            let flvismap = &flash.vis_field;

            let d2 = rpos_raw.distance2(flpos);
            let fl = if is_light_sensitive && d2 < 4.0 {
                // Smooth, distance-only lighting for players/ghosts near light sources
                // Using a 1/r falloff for proximity boost as requested.
                let dist = d2.sqrt();
                // flpower is already adjusted. The 2.0 factor provides a damped proximity boost.
                flpower / (dist + 5.0) * 2.0
            } else {
                let fldir = fldir.with_max_dist(200.0);
                let focus = (fldir.distance() + 0.1).max(6.0) / 20.0;
                let lpos = *flpos + fldir / (100.0 / focus + 20.0);
                let mut lpos = lpos.unrotate_by_dir(&fldir);
                let mut rpos = rpos_raw.unrotate_by_dir(&fldir);
                rpos.x -= lpos.x;
                rpos.y -= lpos.y;
                lpos.x = 0.0;
                lpos.y = 0.0;
                if rpos.x > 0.0 {
                    rpos.x = fastapprox::faster::pow(rpos.x, 1.0 / focus.clamp(1.0, 1.3));
                    rpos.y /= rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }
                if rpos.x < 0.0 {
                    rpos.x = -fastapprox::faster::pow(-rpos.x, (focus / 5.0 + 1.0).clamp(1.0, 4.0));
                    rpos.y *= -rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }

                let dist = (lpos.distance(&rpos) + 0.1)
                    .powf((fldir.distance() / 200.0).clamp(0.2, 1.0).recip());
                let flvis = flvismap[p];
                flpower / (dist + FL_MIN_DST)
                    * flvis.clamp(0.0001, 1.0)
                    * (focus + 0.5).clamp(0.5, 8.0)
            };
            let flsrgba = flcolor.to_srgba();
            lux_fl[0] += fl * flsrgba.red;
            lux_fl[1] += fl * flsrgba.green;
            lux_fl[2] += fl * flsrgba.blue;
            let ld = LightData::from_type(*fltype, fl);
            lightdata = lightdata.add(&ld);
        }
        let ambient_light = 0.0001 + self.tutorial_light_factor * 0.0001;
        self.lf.get(bpos.ndidx()).map(|lf| {
            let r = (lf.lux * lf.color.0 + lux_fl[0] + ambient_light) / self.exposure;
            let g = (lf.lux * lf.color.1 + lux_fl[1] + ambient_light) / self.exposure;
            let b = (lf.lux * lf.color.2 + lux_fl[2] + ambient_light) / self.exposure;

            (
                (
                    tonemapping::artistic_tonemap(r),
                    tonemapping::artistic_tonemap(g),
                    tonemapping::artistic_tonemap(b),
                ),
                lightdata.add(&LightData::from_type(
                    LightType::Visible,
                    lf.lux + ambient_light,
                )),
            )
        })
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

        let mut total_l = 0.0;
        let mut total_v = 0.0;
        let mut total_r = 0.0;
        let mut total_g = 0.0;
        let mut total_b = 0.0;
        let mut total_ld = LightData::default();
        let mut count = 0.0;
        let is_light_sensitive = o_light_sens.is_some();
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
        if count > 0.0 {
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
        }
    }

    pub(crate) fn calc_gamma(&self, lux: f32, tc: f32) -> f32 {
        tonemapping::calc_gamma(
            lux,
            tc,
            self.exposure,
            tonemapping::K_COLD,
            tonemapping::BRIGHTNESS,
        )
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
            self.exposure,
            tonemapping::K_COLD,
            self.tutorial_light_factor,
            tonemapping::DARK_COLOR2,
            dst_color,
            is_tile,
            map_color_alpha,
        )
    }
}
