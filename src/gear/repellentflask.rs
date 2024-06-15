use bevy::prelude::*;
use rand::Rng;

use crate::{board::Position, ghost::GhostSprite, ghost_definitions::GhostType};

use super::{playergear::EquipmentPosition, Gear, GearKind, GearSpriteID, GearUsable};

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
                "Emptying flask...".to_string()
            } else {
                "Activate in ghost room to expel it".to_string()
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

    fn _box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
    fn update(&mut self, gs: &mut super::GearStuff, pos: &Position, _ep: &EquipmentPosition) {
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

        gs.commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    ..default()
                },
                ..default()
            })
            .insert(*pos)
            .insert(Repellent::new(liquid_content));
    }
}

impl RepellentFlask {
    const MAX_QTY: i32 = 200;
    pub fn fill_liquid(&mut self, ghost_type: GhostType) {
        self.liquid_content = Some(ghost_type);
        self.active = false;
        self.qty = Self::MAX_QTY;
    }
    pub fn can_fill_liquid(&self, ghost_type: GhostType) -> bool {
        !(self.liquid_content == Some(ghost_type) && !self.active && self.qty == Self::MAX_QTY)
    }
}

impl From<RepellentFlask> for Gear {
    fn from(value: RepellentFlask) -> Self {
        Gear::new_from_kind(GearKind::RepellentFlask(value))
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Repellent {
    pub class: GhostType,
    pub life: i32,
}

impl Repellent {
    const MAX_LIFE: i32 = 500;
    pub fn new(class: GhostType) -> Self {
        Self {
            class,
            life: Self::MAX_LIFE,
        }
    }

    pub fn life_factor(&self) -> f32 {
        (self.life as f32) / (Self::MAX_LIFE as f32)
    }
}

pub fn repellent_update(
    mut cmd: Commands,
    mut qgs: Query<(&Position, &mut GhostSprite)>,
    mut qrp: Query<(&mut Position, &mut Repellent, Entity), Without<GhostSprite>>,
) {
    let mut rng = rand::thread_rng();
    const SPREAD: f32 = 0.5;
    const SPREAD_SHORT: f32 = 0.05;
    for (mut r_pos, mut rep, entity) in &mut qrp {
        rep.life -= 1;
        if rep.life < 0 {
            cmd.entity(entity).despawn();
            continue;
        }
        let rev_factor = 1.01 - rep.life_factor();

        r_pos.x += rng.gen_range(-SPREAD..SPREAD) * rev_factor
            + rng.gen_range(-SPREAD_SHORT..SPREAD_SHORT);
        r_pos.y += rng.gen_range(-SPREAD..SPREAD) * rev_factor
            + rng.gen_range(-SPREAD_SHORT..SPREAD_SHORT);

        for (g_pos, mut ghost) in &mut qgs {
            let dist = g_pos.distance(&r_pos);
            if dist < 1.0 {
                if ghost.class == rep.class {
                    ghost.repellent_hits += 1;
                } else {
                    ghost.repellent_misses += 1;
                }
                cmd.entity(entity).despawn();
            }
        }
    }
}
