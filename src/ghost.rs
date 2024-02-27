use crate::{
    board::{BoardPosition, Position},
    ghost_definitions::GhostType,
};
use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Debug)]
pub struct GhostSprite {
    pub class: GhostType,
    pub spawn_point: BoardPosition,
    pub target_point: Option<Position>,
    pub repellent_hits: i64,
    pub repellent_misses: i64,
}

#[derive(Component, Debug)]
pub struct GhostBreach;

impl GhostSprite {
    pub fn new(spawn_point: BoardPosition) -> Self {
        let mut rng = rand::thread_rng();
        let ghost_types: Vec<_> = GhostType::all().collect();
        let idx = rng.gen_range(0..ghost_types.len());
        let class = ghost_types[idx];
        warn!("Ghost type: {:?}", class);
        GhostSprite {
            class,
            spawn_point,
            target_point: None,
            repellent_hits: 0,
            repellent_misses: 0,
        }
    }
}

pub fn ghost_movement(
    mut q: Query<(&mut GhostSprite, &mut Position)>,
    roomdb: Res<crate::board::RoomDB>,
    bf: Res<crate::board::BoardData>,
) {
    for (mut ghost, mut pos) in q.iter_mut() {
        if let Some(target_point) = ghost.target_point {
            let mut delta = target_point.delta(*pos);
            let dlen = delta.distance();
            if dlen > 1.0 {
                delta.dx /= dlen.sqrt();
                delta.dy /= dlen.sqrt();
            }
            pos.x += delta.dx / 200.0;
            pos.y += delta.dy / 200.0;
            if dlen < 0.5 {
                ghost.target_point = None;
            }
        } else {
            let mut target_point = ghost.spawn_point.to_position();
            let mut rng = rand::thread_rng();
            let wander: f32 = rng.gen_range(0.0..1.0_f32).powf(6.0) * 12.0 + 0.5;
            let dx: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
            let dy: f32 = (0..5).map(|_| rng.gen_range(-1.0..1.0)).sum();
            let dist: f32 = (0..5).map(|_| rng.gen_range(0.2..wander)).sum();
            let dd = (dx * dx + dy * dy).sqrt() / dist;

            target_point.x = (target_point.x + pos.x * wander) / (1.0 + wander) + dx / dd;
            target_point.y = (target_point.y + pos.y * wander) / (1.0 + wander) + dy / dd;

            let bpos = target_point.to_board_position();
            if roomdb.room_tiles.get(&bpos).is_some()
                && bf
                    .collision_field
                    .get(&bpos)
                    .map(|x| x.ghost_free)
                    .unwrap_or_default()
            {
                ghost.target_point = Some(target_point);
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, ghost_movement);
}
