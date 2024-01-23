use std::{collections::VecDeque, time::Instant};

use crate::{
    behavior::{Behavior, Orientation},
    board::{self, BoardPosition, CollisionFieldData},
    game::{self, GameSound, SoundType},
    materials::CustomMaterial1,
};
use bevy::{prelude::*, utils::HashMap};
use rand::Rng as _;

pub fn compute_visibility(
    vf: &mut HashMap<BoardPosition, f32>,
    cf: &HashMap<BoardPosition, CollisionFieldData>,
    pos_start: &board::Position,
) {
    let mut queue = VecDeque::new();
    let start = pos_start.to_board_position();
    queue.push_front(start.clone());

    *vf.entry(start.clone()).or_default() = 1.0;

    while let Some(pos) = queue.pop_back() {
        let pds = pos.to_position().distance(pos_start);
        let src_f = vf.get(&pos).cloned().unwrap_or_default();
        if !cf.get(&pos).map(|c| c.free).unwrap_or_default() {
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
                let npref = npos.distance(&pos);
                let f = if npds < 1.5 {
                    1.0
                } else {
                    (((npds - pds) / npref - 0.25) / 0.99).clamp(0.0, 1.0)
                };
                let mut dst_f = src_f * f;
                if dst_f < 0.00001 {
                    continue;
                }
                if !vf.contains_key(&npos) {
                    queue.push_front(npos.clone());
                }
                dst_f /= 1.0 + ((npds - 1.5) / 10.0).clamp(0.0, 4.0);
                let entry = vf.entry(npos.clone()).or_insert(dst_f / 2.0);
                *entry = 1.0 - (1.0 - *entry) * (1.0 - dst_f);
            }
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn apply_lighting(
    mut qt: Query<(&board::Position, &mut Sprite)>,
    mut qt2: Query<(&board::Position, &Handle<CustomMaterial1>, &Behavior)>,
    materials1: ResMut<Assets<CustomMaterial1>>,
    qp: Query<(&board::Position, &game::PlayerSprite, &board::Direction)>,
    mut bf: ResMut<board::BoardData>,
    gc: Res<game::GameConfig>,
    qas: Query<(&AudioSink, &GameSound)>,
) {
    const GAMMA_EXP: f32 = 1.5;
    const CENTER_EXP: f32 = 2.3;
    const CENTER_EXP_GAMMA: f32 = 1.9;
    const EYE_SPEED: f32 = 0.5;
    let mut cursor_exp: f32 = 0.001;
    let mut exp_count: f32 = 0.001;
    let mut visibility_field = HashMap::<BoardPosition, f32>::new();
    let mut flashlights = vec![];
    const FLASHLIGHT_ON: bool = true;
    const FLASHLIGHT_POWER: f32 = 1.0;
    // FIXME: This function should not be in level editor
    // FIXME: We need to track the current player of the client (might not be id=1)
    for (pos, player, direction) in qp.iter() {
        if FLASHLIGHT_ON {
            flashlights.push((pos, direction));
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
        compute_visibility(&mut visibility_field, &bf.collision_field, pos);
    }
    // --- ambient sound processing ---
    let total_vis: f32 = visibility_field.iter().map(|(_k, v)| v).sum();
    let house_volume = (120.0 / total_vis).powi(3).tanh().clamp(0.00001, 0.9999);
    let street_volume = (total_vis / 250.0).powi(3).tanh().clamp(0.00001, 0.9999);

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
    cursor_exp = (cursor_exp / CENTER_EXP).powf(CENTER_EXP_GAMMA.recip()) * CENTER_EXP + 0.01;
    if FLASHLIGHT_ON {
        // account for the eye seeing the flashlight on.
        cursor_exp += FLASHLIGHT_POWER.sqrt() / 8.0;
    }

    assert!(cursor_exp.is_normal());
    cursor_exp = cursor_exp / 2.0 + 1.0;

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

    for (pos, mut sprite) in qt.iter_mut() {
        let opacity: f32 = 1.0;
        let bpos = pos.to_board_position();
        let src_color = Color::WHITE;
        let mut dst_color = if let Some(lf) = bf.light_field.get(&bpos) {
            let r: f32 = (bpos.mini_hash() - 0.4) / 50.0;
            let rel_lux = lf.lux / exposure;
            board::compute_color_exposure(rel_lux, r, board::DARK_GAMMA, src_color)
        } else {
            src_color
        };
        dst_color.set_a(opacity.clamp(0.2, 1.0));
        sprite.color = dst_color;
    }

    const VSMALL_PRIME: usize = 13;
    const BIG_PRIME: usize = 95629;
    let mask: usize = rand::thread_rng().gen();
    let lf = &bf.light_field;
    let start = Instant::now();
    let materials1 = materials1.into_inner();
    let mut change_count = 0;
    for (n, (pos, mat, behavior)) in qt2.iter_mut().enumerate() {
        let min_threshold = (((n * BIG_PRIME) ^ mask) % VSMALL_PRIME) as f32 / 10.0;
        let mut opacity: f32 = 1.0;
        let bpos = pos.to_board_position();
        let src_color = Color::WHITE;

        opacity *= src_color.a();

        let bpos_tr = bpos.bottom();
        let bpos_bl = bpos.top();
        let bpos_br = bpos.right();
        let bpos_tl = bpos.left();

        const FL_STRENGTH: f32 = 5.0 * FLASHLIGHT_POWER; // flashlight strength
        const FL_MIN_DST: f32 = 7.0; // minimum distance for flashlight

        let fpos_gamma = |bpos: &BoardPosition| -> Option<f32> {
            let rpos = bpos.to_position();
            let mut lux_fl = 0.0; // lux from flashlight

            for (flpos, fldir) in flashlights.iter() {
                let pdist = flpos.distance(&rpos);
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
                let dist = lpos.distance(&rpos);
                lux_fl +=
                    FL_STRENGTH / (dist * dist + FL_MIN_DST) * (pdist / 5.0 + 0.6).clamp(0.0, 1.0);
            }

            lf.get(bpos).map(|lf| (lf.lux + lux_fl) / exposure)
        };

        let lux_c = fpos_gamma(&bpos).unwrap_or(1.0);
        let mut lux_tr = fpos_gamma(&bpos_tr).unwrap_or(lux_c);
        let mut lux_tl = fpos_gamma(&bpos_tl).unwrap_or(lux_c);
        let mut lux_br = fpos_gamma(&bpos_br).unwrap_or(lux_c);
        let mut lux_bl = fpos_gamma(&bpos_bl).unwrap_or(lux_c);

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
            .min(visibility_field.get(&bpos).copied().unwrap_or_default() * 2.0)
            .min(1.0);

        let mut new_mat = materials1.get(mat).unwrap().clone();
        let orig_mat = new_mat.clone();
        let mut dst_color = src_color; // <- remove brightness calculation for main tile.
        let src_a = new_mat.data.color.a();
        let opacity = opacity.clamp(0.000, 1.0);
        const A_DELTA: f32 = 0.2;
        let new_a = if (src_a - opacity).abs() < A_DELTA {
            opacity
        } else {
            src_a - A_DELTA * (src_a - opacity).signum()
        };
        dst_color.set_a(new_a);
        // const A_SOFT: f32 = 1.0;
        // dst_color.set_a((opacity.clamp(0.000, 1.0) + src_a * A_SOFT) / (1.0 + A_SOFT));
        new_mat.data.color = dst_color;

        const SMOOTH_F: f32 = 1.0;
        let f_gamma = |lux: f32| {
            (fastapprox::faster::pow(lux, board::LIGHT_GAMMA)
                + fastapprox::faster::pow(lux, 1.0 / board::DARK_GAMMA))
                / 2.0
        };
        new_mat.data.gamma = (new_mat.data.gamma * SMOOTH_F + f_gamma(lux_c)) / (1.0 + SMOOTH_F);
        new_mat.data.gtl = (new_mat.data.gtl * SMOOTH_F + f_gamma(lux_tl)) / (1.0 + SMOOTH_F);
        new_mat.data.gtr = (new_mat.data.gtr * SMOOTH_F + f_gamma(lux_tr)) / (1.0 + SMOOTH_F);
        new_mat.data.gbl = (new_mat.data.gbl * SMOOTH_F + f_gamma(lux_bl)) / (1.0 + SMOOTH_F);
        new_mat.data.gbr = (new_mat.data.gbr * SMOOTH_F + f_gamma(lux_br)) / (1.0 + SMOOTH_F);

        let delta = orig_mat.data.delta(&new_mat.data);
        if delta > 0.02 + min_threshold {
            let mat = materials1.get_mut(mat).unwrap();
            mat.data = new_mat.data;
            change_count += 1;
        }
    }
    if mask % 55 == 0 {
        warn!("change_count: {}", &change_count);
        warn!("apply_lighting elapsed: {:?}", start.elapsed());
    }
}
