use std::collections::VecDeque;

use crate::{
    behavior::{Behavior, Orientation},
    board::{self, BoardPosition, CollisionFieldData, Direction, Position},
    game::{self, GameConfig, GameSound, MapUpdate, SoundType, SpriteType},
    gear::{playergear::PlayerGear, GearKind},
    ghost::GhostSprite,
    ghost_definitions::Evidence,
    materials::CustomMaterial1,
    platform::plt::IS_WASM,
    player,
};
use bevy::{prelude::*, utils::HashMap};
use rand::Rng as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Visible,
    Red,
    InfraRedNV,
    UltraViolet,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LightData {
    visible: f32,
    red: f32,
    infrared: f32,
    ultraviolet: f32,
}

impl LightData {
    pub const UNIT_VISIBLE: Self = Self {
        visible: 1.0,
        red: 0.0,
        infrared: 0.0,
        ultraviolet: 0.0,
    };
    pub fn from_type(light_type: LightType, strength: f32) -> Self {
        match light_type {
            LightType::Visible => Self {
                visible: strength,
                ..default()
            },
            LightType::Red => Self {
                red: strength,
                ..default()
            },
            LightType::InfraRedNV => Self {
                infrared: strength,
                ..default()
            },
            LightType::UltraViolet => Self {
                ultraviolet: strength,
                ..default()
            },
        }
    }
    pub fn add(&self, other: &Self) -> Self {
        Self {
            visible: self.visible + other.visible,
            red: self.red + other.red,
            infrared: self.infrared + other.infrared,
            ultraviolet: self.ultraviolet + other.ultraviolet,
        }
    }
    pub fn magnitude(&self) -> f32 {
        let sq_m = self.visible.powi(2)
            + self.red.powi(2)
            + self.infrared.powi(2)
            + self.ultraviolet.powi(2);
        sq_m.sqrt()
    }
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude() + 1.0;
        Self {
            visible: self.visible / mag,
            red: self.red / mag,
            infrared: self.infrared / mag,
            ultraviolet: self.ultraviolet / mag,
        }
    }
}

pub fn compute_visibility(
    vf: &mut HashMap<BoardPosition, f32>,
    cf: &HashMap<BoardPosition, CollisionFieldData>,
    pos_start: &board::Position,
    roomdb: &mut board::RoomDB,
) {
    let mut queue = VecDeque::new();
    let start = pos_start.to_board_position();
    queue.push_front((start.clone(), start.clone()));

    *vf.entry(start.clone()).or_default() = 1.0;

    while let Some((pos, pos2)) = queue.pop_back() {
        let pds = pos.to_position().distance(pos_start);
        let src_f = vf.get(&pos).cloned().unwrap_or_default();
        if !cf
            .get(&pos)
            .map(|c| c.player_free || c.see_through)
            .unwrap_or_default()
        {
            // If the current position analyzed is not free (a wall or out of bounds)
            // then stop extending.
            continue;
        }
        // let neighbors = [pos.left(), pos.top(), pos.bottom(), pos.right()];
        let neighbors = pos.xy_neighbors(1);
        for npos in neighbors {
            if npos == pos {
                continue;
            }
            if cf.contains_key(&npos) {
                let npds = npos.to_position().distance(pos_start);
                let npref = npos.distance(&pos2) / 2.0;
                let f = if npds < 1.5 {
                    1.0
                } else {
                    ((npds - pds) / npref).clamp(0.0, 1.0).powf(2.0)
                };
                let mut dst_f = src_f * f;
                if dst_f < 0.00001 {
                    continue;
                }
                if vf.get(&npos).copied().unwrap_or(-0.01) < 0.0 {
                    queue.push_front((npos.clone(), pos.clone()));
                }
                let k = match roomdb.room_tiles.get(&npos).is_some() {
                    // Decrease view range inside the location
                    true => 3.0,
                    false => {
                        if IS_WASM {
                            4.0
                        } else {
                            7.0
                        }
                    }
                };
                dst_f /= 1.0 + ((npds - 1.5) / k).clamp(0.0, 6.0);
                let entry = vf.entry(npos.clone()).or_insert(dst_f / 2.0);
                *entry = 1.0 - (1.0 - *entry) * (1.0 - dst_f);
            }
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn apply_lighting(
    mut qt: Query<(
        &board::Position,
        &mut Sprite,
        Option<&SpriteType>,
        Option<&GhostSprite>,
    )>,
    mut qt2: Query<
        (
            &board::Position,
            &Handle<CustomMaterial1>,
            &Behavior,
            &mut Visibility,
        ),
        Changed<MapUpdate>,
    >,
    materials1: ResMut<Assets<CustomMaterial1>>,
    qp: Query<(
        &board::Position,
        &player::PlayerSprite,
        &board::Direction,
        &PlayerGear,
    )>,
    mut bf: ResMut<board::BoardData>,
    mut vf: ResMut<board::VisibilityData>,
    gc: Res<game::GameConfig>,
    qas: Query<(&AudioSink, &GameSound)>,
    mut roomdb: ResMut<board::RoomDB>,
    time: Res<Time>,
) {
    const GAMMA_EXP: f32 = 2.4;
    const CENTER_EXP: f32 = 10.0; // Higher values, less blinding light.
    const CENTER_EXP_GAMMA: f32 = 2.25; // Above 1.0, higher the less night vision.
    const EYE_SPEED: f32 = 0.1;
    let mut cursor_exp: f32 = 0.001;
    let mut exp_count: f32 = 0.1;

    vf.visibility_field.clear();

    // FIXME: This "faster" approach causes bugs
    // for v in vf.visibility_field.values_mut() {
    //     *v = -0.01;
    // }

    let mut flashlights = vec![];
    let mut player_pos = Position::new_i64(0, 0, 0);
    let elapsed = time.elapsed_seconds();
    for (pos, player, direction, gear) in qp.iter() {
        let player_flashlight = gear
            .as_vec()
            .into_iter()
            .filter_map(|(g, p)| match &g.kind {
                GearKind::Flashlight(t) => Some((t.power(), t.color(), p, LightType::Visible)),
                GearKind::UVTorch(t) => Some((t.power(), t.color(), p, LightType::UltraViolet)),
                GearKind::RedTorch(t) => Some((t.power(), t.color(), p, LightType::Red)),
                GearKind::Videocam(t) => Some((t.power(), t.color(), p, LightType::InfraRedNV)),
                _ => None,
            });
        for (power, color, p, light_type) in player_flashlight {
            if power > 0.0 {
                use crate::gear::playergear::EquipmentPosition::*;
                let mut fldir = *direction;
                if p == Stowed {
                    fldir = Direction {
                        dx: fldir.dx / 1000.0,
                        dy: fldir.dy / 1000.0,
                        dz: fldir.dz / 1000.0,
                    };
                }

                flashlights.push((pos, fldir, power, color, light_type));
            }
        }

        if player.id != gc.player_id {
            continue;
        }

        let cursor_pos = pos.to_board_position();
        for npos in cursor_pos.xy_neighbors(1) {
            if let Some(lf) = bf.light_field.get(&npos) {
                cursor_exp += lf.lux.powf(GAMMA_EXP);
                exp_count += lf.lux.powf(GAMMA_EXP) / (lf.lux + 0.001);
            }
        }
        compute_visibility(
            &mut vf.visibility_field,
            &bf.collision_field,
            pos,
            &mut roomdb,
        );
        player_pos = *pos;
    }
    // --- ambient sound processing ---

    let total_vis: f32 = vf
        .visibility_field
        .iter()
        .map(|(k, v)| {
            v * match roomdb.room_tiles.get(k).is_some() {
                true => 0.2,
                false => 1.0,
            }
        })
        .sum();
    let house_volume = (20.0 / total_vis).powi(3).tanh().clamp(0.00001, 0.9999) * 6.0;
    let street_volume = (total_vis / 20.0).powi(3).tanh().clamp(0.00001, 0.9999) * 6.0;

    for (sink, gamesound) in qas.iter() {
        const SMOOTH: f32 = 60.0;
        if gamesound.class == SoundType::BackgroundHouse {
            let v = (sink.volume().ln() * SMOOTH + house_volume.ln()) / (SMOOTH + 1.0);
            sink.set_volume(v.exp());
        }
        if gamesound.class == SoundType::BackgroundStreet {
            let v = (sink.volume().ln() * SMOOTH + street_volume.ln()) / (SMOOTH + 1.0);
            sink.set_volume(v.exp());
        }
    }
    // ---
    cursor_exp /= exp_count;
    cursor_exp = (cursor_exp / CENTER_EXP).powf(CENTER_EXP_GAMMA.recip()) * CENTER_EXP + 0.00001;
    // account for the eye seeing the flashlight on.
    // TODO: Account this from the player's perspective as the payer torch might be off but someother player might have it on.
    let fl_total_power: f32 = flashlights.iter().map(|x| x.2).sum();
    cursor_exp += fl_total_power.sqrt() / 8.0;

    assert!(cursor_exp.is_normal());
    cursor_exp += 0.05;

    let exp_f = ((cursor_exp) / bf.current_exposure) / bf.current_exposure_accel.powi(30);
    let max_acc = 1.05;
    bf.current_exposure_accel =
        (bf.current_exposure_accel * 1000.0 + exp_f * EYE_SPEED) / (EYE_SPEED + 1000.0);
    if bf.current_exposure_accel > max_acc {
        bf.current_exposure_accel = max_acc;
    } else if bf.current_exposure_accel.recip() > max_acc {
        bf.current_exposure_accel = max_acc.recip();
    }
    bf.current_exposure_accel = bf.current_exposure_accel.powf(0.99);
    bf.current_exposure *= bf.current_exposure_accel;
    let exposure = bf.current_exposure;

    let mut lightdata_map: HashMap<BoardPosition, LightData> = HashMap::new();

    // 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97
    #[cfg(not(target_arch = "wasm32"))]
    const VSMALL_PRIME: usize = 31;
    #[cfg(target_arch = "wasm32")]
    const VSMALL_PRIME: usize = 97;

    const BIG_PRIME: usize = 95629;
    let mask: usize = rand::thread_rng().gen();
    let lf = &bf.light_field;
    // let start = Instant::now();
    let materials1 = materials1.into_inner();
    // let mut change_count = 0;
    for (n, (pos, mat, behavior, mut vis)) in qt2.iter_mut().enumerate() {
        let min_threshold = (((n * BIG_PRIME) ^ mask) % VSMALL_PRIME) as f32 / 10.0;
        // if min_threshold > 4.5 {
        //     continue;
        // }
        let mut opacity: f32 = 1.0;
        let bpos = pos.to_board_position();

        let bpos_tr = bpos.bottom();
        let bpos_bl = bpos.top();
        let bpos_br = bpos.right();
        let bpos_tl = bpos.left();

        const FL_MIN_DST: f32 = 7.0; // minimum distance for flashlight
                                     // behavior.p.movement.walkable
        let fpos_gamma_color = |bpos: &BoardPosition| -> Option<((f32, f32, f32), LightData)> {
            let rpos = bpos.to_position();
            let mut lux_fl = [0_f32; 3];
            let mut lightdata = LightData::default();

            for (flpos, fldir, flpower, flcolor, fltype) in flashlights.iter() {
                let pdist = flpos.distance(&rpos).powf(2.0);
                let focus = (fldir.distance() - 4.0).max(1.0) / 20.0;
                let lpos = *flpos + *fldir / (100.0 / focus);
                let mut lpos = lpos.unrotate_by_dir(fldir);
                let mut rpos = rpos.unrotate_by_dir(fldir);
                rpos.x -= lpos.x;
                rpos.y -= lpos.y;
                lpos.x = 0.0;
                lpos.y = 0.0;
                if rpos.x > 0.0 {
                    rpos.x = fastapprox::faster::pow(rpos.x, 1.0 / focus.clamp(1.0, 1.1));
                    rpos.y /= rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }
                if rpos.x < 0.0 {
                    rpos.x = -fastapprox::faster::pow(-rpos.x, (focus / 5.0 + 1.0).clamp(1.0, 3.0));
                    rpos.y *= -rpos.x * (focus - 1.0).clamp(0.0, 10.0) / 30.0 + 1.0;
                }
                let dist = lpos
                    .distance(&rpos)
                    .powf(fldir.distance().clamp(0.01, 30.0).recip().clamp(1.0, 3.0));
                let fl = flpower / (dist * dist + FL_MIN_DST) * (pdist / 5.0).clamp(0.0, 1.0);
                lux_fl[0] += fl * flcolor.r();
                lux_fl[1] += fl * flcolor.g();
                lux_fl[2] += fl * flcolor.b();

                let ld = LightData::from_type(*fltype, fl);
                lightdata = lightdata.add(&ld);
            }
            const AMBIENT_LIGHT: f32 = 0.0001;
            lf.get(bpos).map(|lf| {
                (
                    (
                        (lf.lux + lux_fl[0] + AMBIENT_LIGHT) / exposure,
                        (lf.lux + lux_fl[1] + AMBIENT_LIGHT) / exposure,
                        (lf.lux + lux_fl[2] + AMBIENT_LIGHT) / exposure,
                    ),
                    lightdata.add(&LightData::from_type(
                        LightType::Visible,
                        lf.lux + AMBIENT_LIGHT,
                    )),
                )
            })
        };

        let fpos_gamma = |bpos: &BoardPosition| -> Option<f32> {
            let gcolor = fpos_gamma_color(bpos);
            gcolor.map(|((r, g, b), _)| (r + g + b) / 3.0)
        };
        let ((r, g, b), light_data) =
            fpos_gamma_color(&bpos).unwrap_or(((1.0, 1.0, 1.0), LightData::UNIT_VISIBLE));
        if behavior.p.movement.walkable {
            let ld = light_data.normalize();
            lightdata_map.insert(bpos.clone(), ld);
        }
        let max_color = r.max(g).max(b).max(0.01) + 0.01;
        let src_color = Color::rgb(r / max_color, g / max_color, b / max_color);
        let l = src_color.l().max(0.0001).powf(1.5);

        let mut lux_c = fpos_gamma(&bpos).unwrap_or(1.0) / l;
        let mut lux_tr = fpos_gamma(&bpos_tr).unwrap_or(lux_c) / l;
        let mut lux_tl = fpos_gamma(&bpos_tl).unwrap_or(lux_c) / l;
        let mut lux_br = fpos_gamma(&bpos_br).unwrap_or(lux_c) / l;
        let mut lux_bl = fpos_gamma(&bpos_bl).unwrap_or(lux_c) / l;

        match behavior.obsolete_occlusion_type() {
            Orientation::None => {}
            Orientation::XAxis => {
                lux_tl = lux_c;
                lux_br = lux_c;
            }
            Orientation::YAxis => {
                lux_tr = lux_c;
                lux_bl = lux_c;
            }
            Orientation::Both => {
                lux_tl = lux_c;
                lux_br = lux_c;
                lux_tr = lux_c;
                lux_bl = lux_c;
            }
        }
        let mut dst_color = {
            let r: f32 = (bpos.mini_hash() - 0.4) / 50.0;
            board::compute_color_exposure(lux_c, r, board::DARK_GAMMA, src_color)
        };
        dst_color.set_a(opacity.clamp(0.6, 1.0));

        opacity = opacity
            .min(vf.visibility_field.get(&bpos).copied().unwrap_or_default() * 2.0)
            .clamp(0.0, 1.0);

        let mut new_mat = materials1.get(mat).unwrap().clone();
        let orig_mat = new_mat.clone();
        let mut dst_color = src_color; // <- remove brightness calculation for main tile.
        let src_a = new_mat.data.color.a();
        let opacity = opacity.clamp(0.000, 1.0);
        const A_DELTA: f32 = 0.05;
        let new_a = if (src_a - opacity).abs() < A_DELTA {
            opacity
        } else {
            src_a - A_DELTA * (src_a - opacity).signum()
        };
        dst_color.set_a(new_a);
        // Sound field visualization:

        let f_gamma = |lux: f32| {
            (fastapprox::faster::pow(lux, board::LIGHT_GAMMA)
                + fastapprox::faster::pow(lux, 1.0 / board::DARK_GAMMA))
                / 2.0
        };
        const K_COLD: f32 = 0.5;
        let cold_f = (1.0 - (lux_c / K_COLD).tanh()) * 2.0;

        const DARK_COLOR: Color = Color::rgba(0.247 / 1.5, 0.714 / 1.5, 0.878, 1.0);
        const DARK_COLOR2: Color = Color::rgba(0.03, 0.336, 0.444, 1.0);
        let exp_color =
            ((-(exposure + 0.0001).ln() / 2.0 - 1.5 + cold_f).tanh() + 0.5).clamp(0.0, 1.0);
        let dark = lerp_color(Color::BLACK, DARK_COLOR, exp_color / 16.0);
        let dark2 = lerp_color(
            Color::WHITE,
            DARK_COLOR2,
            exp_color / f_gamma(lux_c).clamp(1.0, 300.0),
        );
        new_mat.data.ambient_color = dark.with_a(0.0);
        // const A_SOFT: f32 = 1.0;
        // dst_color.set_a((opacity.clamp(0.000, 1.0) + src_a * A_SOFT) / (1.0 + A_SOFT));
        new_mat.data.color =
            Color::rgba_from_array(dst_color.rgba_to_vec4() * dark2.rgba_to_vec4());

        const BRIGHTNESS: f32 = 1.01;
        let tint_comp = (new_mat.data.color.l() + 0.01).recip() + exp_color;
        let smooth_f: f32 = new_mat.data.color.a().sqrt() * 10.0 + 0.0001;

        let gamma_mean = |a: f32, b: f32| {
            (a * smooth_f
                + f_gamma(
                    b * BRIGHTNESS * (1.0 + cold_f + (exp_color * 2.0).powi(2))
                        + (tint_comp - 1.0 + cold_f * 2.0 + (exp_color * 2.0).powi(2))
                            / (10.0 + exposure + b),
                )
                + exp_color / 40.0)
                / (1.0 + smooth_f)
        };
        lux_c = (lux_c * 4.0 + lux_tl + lux_tr + lux_bl + lux_br) / 8.0;
        new_mat.data.gamma = gamma_mean(new_mat.data.gamma, lux_c);
        new_mat.data.gtl = gamma_mean(new_mat.data.gtl, (lux_tl + lux_c) / 2.0);
        new_mat.data.gtr = gamma_mean(new_mat.data.gtr, (lux_tr + lux_c) / 2.0);
        new_mat.data.gbl = gamma_mean(new_mat.data.gbl, (lux_bl + lux_c) / 2.0);
        new_mat.data.gbr = gamma_mean(new_mat.data.gbr, (lux_br + lux_c) / 2.0);

        const DEBUG_SOUND: bool = false;
        if DEBUG_SOUND {
            if let Some(sf) = bf.sound_field.get(&bpos) {
                let l: f32 = sf.iter().map(|x| x.length() + 0.01).sum();
                if l > 0.0001 {
                    new_mat.data.gamma = 2.0;
                    new_mat.data.color = Color::rgb(1.0, l / 4.0, l / 16.0);
                }
            }
        }
        let invisible = new_mat.data.color.a() < 0.01 || behavior.p.display.disable;
        let new_vis = if invisible {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        *vis = new_vis;

        let delta = orig_mat.data.delta(&new_mat.data);
        let thr = if IS_WASM { 0.2 } else { 0.02 };
        if !invisible && delta > thr + min_threshold {
            let mat = materials1.get_mut(mat).unwrap();
            mat.data = new_mat.data;
            // change_count += 1;
        }
    }

    // Light ilumination for sprites on map that aren't part of the map (player, ghost, van, ghost breach)

    for (pos, mut sprite, o_type, o_gs) in qt.iter_mut() {
        let stype = o_type.cloned().unwrap_or_default();
        let bpos = pos.to_board_position();
        let Some(ld) = lightdata_map.get(&bpos).cloned() else {
            // If the given cell was not selected for update, skip updating its color (otherwise it can blink)
            continue;
        };
        let mut opacity: f32 = 1.0
            * vf.visibility_field
                .get(&bpos)
                .copied()
                .unwrap_or_default()
                .clamp(0.0, 1.0);
        opacity = (opacity.powf(0.5) * 2.0 - 0.1).clamp(0.0001, 1.0);
        let src_color = Color::WHITE;
        let mut dst_color = if let Some(lf) = bf.light_field.get(&bpos) {
            let r: f32 = (bpos.mini_hash() - 0.4) / 50.0;
            let mut rel_lux = lf.lux / exposure;
            if stype == SpriteType::Ghost {
                rel_lux /= 2.0;
            }
            if stype == SpriteType::Player {
                rel_lux *= 1.1;
                rel_lux += 0.1;
            }

            board::compute_color_exposure(rel_lux, r, board::DARK_GAMMA, src_color)
        } else {
            src_color
        };
        let mut smooth: f32 = 20.0;
        if stype == SpriteType::Ghost {
            let Some(gs) = o_gs else {
                continue;
            };
            if gs.hunt_target {
                dst_color = Color::RED;
            } else {
                opacity *= dst_color.l().clamp(0.7, 1.0);
                // Make the ghost oscilate to increase visibility:
                let osc1 = (elapsed * 1.0).sin() * 0.25 + 0.75;
                let osc2 = (elapsed * 1.15).cos() * 0.5 + 0.5;

                opacity = opacity.min(osc1 + 0.2) / (1.0 + gs.warp / 5.0);
                let l = (dst_color.l() + osc2) / 2.0;
                dst_color.set_l(l);
                let r = dst_color.r();
                let g = dst_color.g();
                let e_uv = if bf.evidences.contains(&Evidence::UVEctoplasm) {
                    ld.ultraviolet * 3.0
                } else {
                    0.0
                };
                let e_rl = if bf.evidences.contains(&Evidence::RLPresence) {
                    ld.red * 3.0
                } else {
                    0.0
                };

                dst_color.set_l(l * ld.visible);
                dst_color.set_r(r * ld.visible + e_rl);
                dst_color.set_g(g * ld.visible + e_uv + e_rl / 2.0);
            }
            smooth = 1.0;
            dst_color = lerp_color(sprite.color, dst_color, 0.04);
        }
        if stype == SpriteType::Breach {
            smooth = 2.0;
            let e_nv = if bf.evidences.contains(&Evidence::FloatingOrbs) {
                ld.infrared * 3.0
            } else {
                0.0
            };
            opacity *= ((dst_color.l() / 3.0) + e_nv / 4.0).clamp(0.0, 0.5);
            let l = dst_color.l();
            // Make the breach oscilate to increase visibility:
            let osc1 = ((elapsed * 0.62).sin() * 10.0 + 8.0).tanh() * 0.5 + 0.5;

            dst_color.set_l(((l * ld.visible + e_nv) * osc1).clamp(0.0, 0.99));
        }
        let mut old_a = sprite.color.a().clamp(0.0001, 1.0);
        if stype == SpriteType::Other {
            const MAX_DIST: f32 = 8.0;
            let dist = pos.distance(&player_pos);
            if dist < MAX_DIST {
                let delta_z = pos.to_screen_coord().z - player_pos.to_screen_coord().z;
                if delta_z > 0.0 {
                    old_a /= 1.1;
                }
            }
        }

        dst_color.set_a((opacity + old_a * smooth) / (smooth + 1.0));
        sprite.color = dst_color;
    }

    for (bpos, ld) in lightdata_map.into_iter() {
        bf.light_field.entry(bpos).and_modify(|l| l.additional = ld);
    }
    // if mask % 55 == 0 {
    //     warn!("change_count: {}", &change_count);
    //     warn!("apply_lighting elapsed: {:?}", start.elapsed());
    // }
}

pub fn mark_for_update(
    time: Res<Time>,
    gc: Res<GameConfig>,
    qp: Query<(&board::Position, &player::PlayerSprite)>,
    mut qt2: Query<(&board::Position, &Visibility, &mut MapUpdate)>,
) {
    let mut player_pos = Position::new_i64(0, 0, 0);
    for (pos, player) in qp.iter() {
        if player.id != gc.player_id {
            continue;
        }
        player_pos = *pos;
    }
    let now = time.elapsed_seconds();
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    let mut small_rng = SmallRng::from_entropy();

    for (pos, vis, mut upd) in qt2.iter_mut() {
        let r: f32 = small_rng.gen_range(0.0..1.01);
        let dst = pos.distance_taxicab(&player_pos);
        let min_dst = 5.0
            + if vis == Visibility::Hidden {
                r.powi(10) * 20.0
            } else {
                r.powi(10) * 100.0
            };
        if dst < min_dst {
            upd.last_update = now;
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, (mark_for_update, apply_lighting).chain());
}

pub fn lerp_color(start: Color, end: Color, t: f32) -> Color {
    let k = start.as_rgba_f32();
    let l = end.as_rgba_f32();
    let (sr, sg, sb, sa) = (k[0], k[1], k[2], k[3]);
    let (er, eg, eb, ea) = (l[0], l[1], l[2], l[3]);
    let r = sr + (er - sr) * t;
    let g = sg + (eg - sg) * t;
    let b = sb + (eb - sb) * t;
    let a = sa + (ea - sa) * t;
    Color::rgba(r, g, b, a)
}
