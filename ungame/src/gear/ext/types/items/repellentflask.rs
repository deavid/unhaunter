use uncore::{
    components::{
        board::{boardposition::BoardPosition, direction::Direction, position::Position},
        game::GameSprite,
    },
    types::{gear::equipmentposition::EquipmentPosition, ghost::types::GhostType},
};

use crate::{difficulty::CurrentDifficulty, ghost::GhostSprite, maplight::MapColor};

use super::{Gear, GearKind, GearSpriteID, GearUsable};
use bevy::{color::palettes::css, prelude::*, utils::hashbrown::HashMap};
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
        "Unhaunter Repellent"
    }

    fn get_description(&self) -> &'static str {
        "Crafted in the van, specifically targeting a single ghost type to be effective enough to expel a ghost."
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = match self.liquid_content {
            Some(x) => format!("\nAnti-{} essence", x.name()),
            None => "Empty".to_string(),
        };
        let msg = if self.liquid_content.is_some() {
            if self.active {
                "Emptying flask... get close to the ghost!".to_string()
            } else {
                "Make the ghost absorb the repellent to expel it".to_string()
            }
        } else {
            "Flask must be filled on the van".to_string()
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
        let mut rng = rand::thread_rng();
        let spread: f32 = if matches!(ep, EquipmentPosition::Deployed) {
            0.1
        } else {
            0.4
        };
        pos.x += rng.gen_range(-spread..spread);
        pos.y += rng.gen_range(-spread..spread);
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
    pub life: i32,
    pub dir: Direction,
}

impl Repellent {
    const MAX_LIFE: i32 = 1500;

    pub fn new(class: GhostType) -> Self {
        Self {
            class,
            life: Self::MAX_LIFE,
            dir: Direction::zero(),
        }
    }

    pub fn life_factor(&self) -> f32 {
        (self.life as f32) / (Self::MAX_LIFE as f32)
    }
}

pub fn repellent_update(
    mut cmd: Commands,
    mut qgs: Query<(&Position, &mut GhostSprite)>,
    mut qrp: Query<(&mut Position, &mut Repellent, &mut MapColor, Entity), Without<GhostSprite>>,
    difficulty: Res<CurrentDifficulty>,
) {
    let mut rng = rand::thread_rng();
    const SPREAD: f32 = 0.1;
    const SPREAD_SHORT: f32 = 0.02;
    let mut pressure: HashMap<BoardPosition, f32> = HashMap::new();
    const RADIUS: f32 = 0.7;
    for (r_pos, rep, _, _) in &qrp {
        let bpos = r_pos.to_board_position();
        for nb in bpos.xy_neighbors(3) {
            let dist2 = (nb.to_position_center().distance(r_pos) * RADIUS).powi(2);
            let exponent: f32 = -0.5 * dist2;
            let gauss = exponent.exp();
            let life = 1.001 - rep.life_factor();
            *pressure.entry(nb).or_default() += gauss * life;
        }
    }
    for (mut r_pos, mut rep, mut mapcolor, entity) in &mut qrp {
        rep.life -= 1;
        if rep.life < 0 {
            cmd.entity(entity).despawn();
            continue;
        }
        let rev_factor = 1.01 - rep.life_factor();
        mapcolor
            .color
            .set_alpha(rep.life_factor().sqrt() / 4.0 + 0.01);
        let bpos = r_pos.to_board_position();
        let mut total_force = Direction::zero();
        for nb in bpos.xy_neighbors(3) {
            let npos = nb.to_position_center();
            let dist2 = (npos.distance(&r_pos) * RADIUS).powi(2);
            let exponent: f32 = -0.5 * dist2;
            let gauss = exponent.exp();
            let vector = r_pos.delta(npos);
            let psi = *pressure.entry(nb).or_default();
            let mut vector_scaled = vector.normalized().mul(psi * gauss);
            vector_scaled.dz = 0.0;
            total_force = total_force + vector_scaled;
        }

        // total_force = total_force.normalized().mul(total_force.distance().sqrt());
        const PRESSURE_FORCE_SCALE: f32 = 1e-4;
        rep.dir = rep.dir.add(total_force.mul(PRESSURE_FORCE_SCALE)).mul(0.97);
        r_pos.x += rng.gen_range(-SPREAD..SPREAD) * rev_factor
            + rng.gen_range(-SPREAD_SHORT..SPREAD_SHORT)
            + rep.dir.dx;
        r_pos.y += rng.gen_range(-SPREAD..SPREAD) * rev_factor
            + rng.gen_range(-SPREAD_SHORT..SPREAD_SHORT)
            + rep.dir.dy;
        r_pos.z += (rng.gen_range(-SPREAD..SPREAD) * rev_factor
            + rng.gen_range(-SPREAD_SHORT..SPREAD_SHORT))
            / 10.0;
        r_pos.z = (r_pos.z * 100.0 + 0.5 * rep.life_factor()) / 101.0;
        for (g_pos, mut ghost) in &mut qgs {
            let dist = g_pos.distance(&r_pos);
            if dist < 1.5 {
                if ghost.class == rep.class {
                    ghost.repellent_hits_frame += 1.2 / (dist + 1.0);
                } else {
                    ghost.repellent_misses_frame += 1.2 / (dist + 1.0);
                }
                rep.life -= 20;
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
}
