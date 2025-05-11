use bevy::utils::HashSet;
use fastapprox::faster;
use ndarray::Array3;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::mapcolor::MapColor;
use uncore::components::board::{direction::Direction, position::Position};
use uncore::metric_recorder::SendMetric;
use uncore::random_seed;
use uncore::resources::board_data::BoardData;
use uncore::{
    components::{game::GameSprite, ghost_sprite::GhostSprite},
    difficulty::CurrentDifficulty,
    types::{gear::equipmentposition::EquipmentPosition, ghost::types::GhostType},
};

use crate::metrics;

use super::{Gear, GearKind, GearSpriteID, GearUsable};
use bevy::{color::palettes::css, prelude::*};
use rand::Rng;
use std::ops::{Add, Mul};

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct RepellentFlask {
    pub liquid_content: Option<GhostType>,
    pub active: bool,
    pub qty: i32,
}

impl GearUsable for RepellentFlask {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.liquid_content.is_some() {
            true => GearSpriteID::RepelentFlaskFull,
            false => GearSpriteID::RepelentFlaskEmpty,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Repellent"
    }

    fn get_description(&self) -> &'static str {
        "Crafted in the van, specifically targeting a single ghost type to be effective enough to expel a ghost."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = match self.liquid_content {
            Some(x) => format!("Anti-{}", x.name()),
            None => "Empty".to_string(),
        };
        let msg = if self.liquid_content.is_some() {
            if self.active {
                "Emptying flask...\nGet close to the ghost!".to_string()
            } else {
                "Flask ready.\nActivate near the Ghost.".to_string()
            }
        } else {
            "Flask empty.\nMust be filled on the van".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self, _gs: &mut super::GearStuff) {
        if self.liquid_content.is_none() {
            return;
        }
        self.active = true;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }

    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, ep: &EquipmentPosition) {
        if !self.active {
            return;
        }
        let mut rng = random_seed::rng();
        if rng.random_range(0.0..1.0) > 0.5 {
            // Reduce the amount of particles emitted. Also reduces the speed of depletion.
            return;
        }

        if self.qty == Self::MAX_QTY {
            gs.summary.repellent_used_amt += 1;
        }
        self.qty -= 1;
        if self.qty <= 0 {
            self.qty = 0;
            self.active = false;
            self.liquid_content = None;
            return;
        }
        let Some(liquid_content) = self.liquid_content else {
            self.qty = 0;
            self.active = false;
            return;
        };
        let mut pos = *pos;
        pos.z += 0.2;
        let spread: f32 = if matches!(ep, EquipmentPosition::Deployed) {
            0.1
        } else {
            0.4
        };
        pos.x += rng.random_range(-spread..spread);
        pos.y += rng.random_range(-spread..spread);
        gs.commands
            .spawn(Sprite {
                color: Color::NONE,
                ..default()
            })
            .insert(pos)
            .insert(GameSprite)
            .insert(MapColor {
                color: css::YELLOW.with_alpha(0.3).with_blue(0.02).into(),
            })
            .insert(Repellent::new(liquid_content));
    }

    fn can_fill_liquid(&self, ghost_type: GhostType) -> bool {
        !(self.liquid_content == Some(ghost_type) && !self.active && self.qty == Self::MAX_QTY)
    }
    fn do_fill_liquid(&mut self, ghost_type: GhostType) {
        self.liquid_content = Some(ghost_type);
        self.active = false;
        self.qty = Self::MAX_QTY;
    }
}

impl RepellentFlask {
    const MAX_QTY: i32 = 400;
}

impl From<RepellentFlask> for Gear {
    fn from(value: RepellentFlask) -> Self {
        Gear::new_from_kind(GearKind::RepellentFlask, value.box_clone())
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Repellent {
    pub class: GhostType,
    pub life: f32,
    pub dir: Direction,
}

impl Repellent {
    const MAX_LIFE: f32 = 30.0;

    pub fn new(class: GhostType) -> Self {
        Self {
            class,
            life: Self::MAX_LIFE,
            dir: Direction::zero(),
        }
    }

    pub fn life_factor(&self) -> f32 {
        self.life / Self::MAX_LIFE
    }
}

pub fn repellent_update(
    mut cmd: Commands,
    mut qgs: Query<(&Position, &mut GhostSprite)>,
    mut qrp: Query<(&mut Position, &mut Repellent, &mut MapColor, Entity), Without<GhostSprite>>,
    bf: Res<BoardData>,
    difficulty: Res<CurrentDifficulty>,
    mut pressure_base: Local<Array3<f32>>,
    mut positions: Local<Array3<Vec<Vec3>>>,
    time: Res<Time>,
) {
    let measure = metrics::REPELLENT_UPDATE.time_measure();

    let mut rng = random_seed::rng();
    let dt = time.delta_secs();
    const SPREAD: f32 = 0.1;
    const SPREAD_SHORT: f32 = 0.02;
    if pressure_base.dim() != bf.map_size {
        *pressure_base = Array3::from_elem(bf.map_size, 0.0);
    }

    pressure_base.indexed_iter_mut().for_each(|(p, v)| {
        *v = if bf.collision_field[p].player_free {
            20.0
        } else {
            0.0
        }
    });
    if positions.dim() != bf.map_size {
        *positions = Array3::from_elem(bf.map_size, Vec::with_capacity(8));
    }
    positions.iter_mut().for_each(|v| v.clear());

    const RADIUS: f32 = 0.7;
    let mut p_set = HashSet::with_capacity(1024);

    for (r_pos, rep, _, _) in &qrp {
        let bpos = r_pos.to_board_position();
        let life = 1.001 - rep.life_factor();
        let nidx = bpos.ndidx();
        let Some(pres) = pressure_base.get_mut(nidx) else {
            continue;
        };
        *pres += life;
        p_set.insert(nidx);
        positions[nidx].push(r_pos.to_vec3());
    }
    let mut pressure: Array3<f32> = Array3::from_elem(bf.map_size, 0.0);
    for &p in p_set.iter() {
        let pres = pressure_base[p];
        if !(0.0001..=19.0).contains(&pres) {
            // Skip cells that don't have anything on them or are walls
            continue;
        }
        let bpos = BoardPosition::from_ndidx(p);
        for nb in bpos.iter_xy_neighbors(2, bf.map_size) {
            let dist2 = nb.distance2(&bpos) * RADIUS;
            let exponent: f32 = -0.5 * dist2;
            let gauss = faster::exp(exponent);
            pressure[nb.ndidx()] += gauss * pres;
        }
    }

    for (mut r_pos, mut rep, mut mapcolor, entity) in &mut qrp {
        rep.life -= dt;
        if rep.life < 0.0 {
            cmd.entity(entity).despawn();
            continue;
        }
        let life_factor = rep.life_factor();
        let rev_factor = 1.01 - life_factor;
        mapcolor.color.set_alpha(life_factor.cbrt() / 2.0 + 0.01);
        let bpos = r_pos.to_board_position();
        let rr_pos = Position {
            x: r_pos.x + rng.random_range(-0.5..0.5),
            y: r_pos.y + rng.random_range(-0.5..0.5),
            z: r_pos.z + rng.random_range(-0.5..0.5),
            global_z: r_pos.global_z,
        };
        let ndidx = bpos.ndidx();

        let mut total_force = Direction::zero();
        for nb in bpos.iter_xy_neighbors(2, bf.map_size) {
            let npos = nb.to_position();
            let vector = rr_pos.delta(npos);
            let dist2 = vector.distance2();
            let psi = pressure[nb.ndidx()] / (0.2 + dist2) * 3.0;

            total_force.dx += vector.dx * psi;
            total_force.dy += vector.dy * psi;
        }
        let v_pos = r_pos.to_vec3();
        for &s_p in positions[ndidx].iter() {
            let dist2 = v_pos.distance_squared(s_p) + 0.1;
            let delta = v_pos - s_p;
            let force = 4.0 * delta / dist2;
            total_force.dx += force.x;
            total_force.dy += force.y;
        }
        total_force.dx += rng.random_range(-0.1..0.1);
        total_force.dy += rng.random_range(-0.1..0.1);

        // total_force = total_force.normalized().mul(total_force.distance().sqrt());
        const PRESSURE_FORCE_SCALE: f32 = 1e-5;
        rep.dir = rep
            .dir
            .add(total_force.mul(PRESSURE_FORCE_SCALE))
            .mul(0.999);

        for nb in bpos.iter_xy_neighbors(1, bf.map_size) {
            let coll_tile_data = &bf.collision_field[nb.ndidx()];
            if !coll_tile_data.player_free && !coll_tile_data.see_through {
                // Collision with walls
                let wall_pos = nb.to_position();
                let delta = r_pos.delta(wall_pos);
                let dist2 = delta.distance2() + 0.2;
                let norm = delta.normalized();
                let recip = dist2.recip();
                let force = recip * 0.001;
                if bpos == nb {
                    rep.dir.dx *= 0.8;
                    rep.dir.dy *= 0.8;
                }
                rep.dir.dx += norm.dx * force;
                rep.dir.dy += norm.dy * force;
            }
        }
        r_pos.x += rng.random_range(-SPREAD..SPREAD) * rev_factor
            + rng.random_range(-SPREAD_SHORT..SPREAD_SHORT)
            + rep.dir.dx;
        r_pos.y += rng.random_range(-SPREAD..SPREAD) * rev_factor
            + rng.random_range(-SPREAD_SHORT..SPREAD_SHORT)
            + rep.dir.dy;
        r_pos.z += (rng.random_range(-SPREAD..SPREAD) * rev_factor
            + rng.random_range(-SPREAD_SHORT..SPREAD_SHORT))
            / 10.0;

        // Get the base floor height (integer part of z)
        let floor_height = r_pos.z.floor();
        // Apply the floating effect but preserve the floor level
        let particle_height = (r_pos.z - floor_height) * 100.0 + 0.5 * rep.life_factor();
        r_pos.z = floor_height + (particle_height / 101.0);

        if r_pos
            .to_board_position()
            .ndidx_checked_margin(bf.map_size)
            .is_none()
        {
            rep.life = 0.0;
        }
        for (g_pos, mut ghost) in &mut qgs {
            let dist = g_pos.distance(&r_pos);
            if dist < 1.5 {
                if ghost.class == rep.class {
                    ghost.repellent_hits_frame += dt * 183.2 / (dist + 1.0);
                } else {
                    ghost.repellent_misses_frame += dt * 183.2 / (dist + 1.0);
                }
                rep.life -= 20.0 * dt;
                // cmd.entity(entity).despawn();
            }
        }
    }
    for (_pos, mut ghost) in &mut qgs {
        if ghost.repellent_hits_frame > 1.0 {
            ghost.repellent_hits += 1;
            ghost.repellent_hits_frame = 0.0;
            ghost.rage += 0.6 * difficulty.0.ghost_rage_likelihood;
        }
        if ghost.repellent_misses_frame > 1.0 {
            ghost.repellent_misses += 1;
            ghost.repellent_misses_frame = 0.0;
            ghost.rage += 0.6 * difficulty.0.ghost_rage_likelihood;
        }
    }

    measure.end_ms();
}
