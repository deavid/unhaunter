use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct Anchors {
    pub base: Vec2,
    pub grid1x1: Vec2,
    pub grid1x1x4: Vec2,
    pub character: Vec2,
}

impl Anchors {
    /// Computes the anchors for the given sprite in pixels
    pub fn calc(pos_x: i32, pos_y: i32, size_x: i32, size_y: i32) -> Vec2 {
        Anchors::calc_f32(pos_x as f32, pos_y as f32, size_x as f32, size_y as f32)
    }

    /// Computes the anchors for the given sprite in pixels, f32 variant
    pub fn calc_f32(pos_x: f32, pos_y: f32, size_x: f32, size_y: f32) -> Vec2 {
        let x = pos_x / size_x - 0.5;
        let y = 0.5 - pos_y / size_y;
        Vec2::new(x, y)
    }
}
