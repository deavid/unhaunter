use bevy::{math::Vec2, utils::HashMap};

use crate::components::board::boardposition::BoardPosition;

#[derive(Debug, Clone, Default)]
pub struct MiasmaGrid {
    pub pressure_field: HashMap<BoardPosition, f32>,
    pub velocity_field: HashMap<BoardPosition, Vec2>,
    pub room_modifiers: HashMap<String, f32>, // Room ID -> Modifier
}
